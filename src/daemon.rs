// ============================================================================
// stick - 데몬 프로세스 관리 모듈
// macOS: launchd (plist), Linux: systemd (unit file)
// ============================================================================

use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// 현재 OS에 맞는 서비스 매니저 판별
enum ServiceManager {
    Launchd,  // macOS
    Systemd,  // Linux
}

/// 실행 환경의 서비스 매니저 감지
fn detect_service_manager() -> Result<ServiceManager> {
    if cfg!(target_os = "macos") {
        Ok(ServiceManager::Launchd)
    } else if cfg!(target_os = "linux") {
        // systemctl 존재 여부 확인
        let has_systemctl = Command::new("which")
            .arg("systemctl")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if has_systemctl {
            Ok(ServiceManager::Systemd)
        } else {
            Err(anyhow::anyhow!(
                "systemd를 찾을 수 없습니다. 현재 Linux 환경에서 systemd가 필요합니다."
            ))
        }
    } else if cfg!(target_os = "windows") {
        Err(anyhow::anyhow!(
            "💡 Windows 환경은 파일명이 항상 자소 결합(NFC) 형태로 자동 저장되므로 실시간 감시 백그라운드 데몬이 필요하지 않습니다.\n\n\
             맥 사용자가 공유하여 이미 깨진 파일들을 터치 한 번에 깔끔히 치료하려면 아래 명령어를 사용해보세요!\n\
             👉 stick scan\n\n\
             스캔 대상 폴더 및 규칙을 편리하게 추가하려면:\n\
             👉 stick config"
        ))
    } else {
        Err(anyhow::anyhow!(
            "지원하지 않는 운영체제입니다. macOS, Linux, Windows 환경에서 사용할 수 있습니다."
        ))
    }
}

// ── launchd (macOS) ──────────────────────────────────────────────────────

/// launchd plist 파일 경로
/// ~/Library/LaunchAgents/com.stick.agent.plist
fn launchd_plist_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("홈 디렉토리를 찾을 수 없습니다")?;
    Ok(home
        .join("Library")
        .join("LaunchAgents")
        .join("com.stick.agent.plist"))
}

/// stick 실행 파일의 절대 경로를 찾는 함수
fn find_stick_binary() -> Result<PathBuf> {
    // 현재 실행 중인 바이너리 경로 사용
    let current_exe = std::env::current_exe()
        .context("현재 실행 파일 경로를 가져올 수 없습니다")?;
    Ok(current_exe)
}

/// launchd plist 내용 생성
fn generate_launchd_plist(stick_binary: &str, log_dir: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.stick.agent</string>

    <key>ProgramArguments</key>
    <array>
        <string>{binary}</string>
        <string>watch</string>
    </array>

    <key>RunAtLoad</key>
    <true/>

    <key>KeepAlive</key>
    <true/>

    <key>StandardOutPath</key>
    <string>{log_dir}/stick_stdout.log</string>

    <key>StandardErrorPath</key>
    <string>{log_dir}/stick_stderr.log</string>

    <key>ProcessType</key>
    <string>Background</string>
</dict>
</plist>"#,
        binary = stick_binary,
        log_dir = log_dir,
    )
}

/// launchd 서비스 등록 및 시작
fn launchd_start(log_dir: &str) -> Result<()> {
    let plist_path = launchd_plist_path()?;
    let stick_binary = find_stick_binary()?;

    // LaunchAgents 디렉토리 확인
    if let Some(parent) = plist_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("LaunchAgents 디렉토리 생성 실패: {:?}", parent))?;
    }

    // 로그 디렉토리 생성
    fs::create_dir_all(log_dir)
        .with_context(|| format!("로그 디렉토리 생성 실패: {}", log_dir))?;

    // plist 파일 생성
    let plist_content =
        generate_launchd_plist(&stick_binary.to_string_lossy(), log_dir);
    fs::write(&plist_path, &plist_content)
        .with_context(|| format!("plist 파일 생성 실패: {:?}", plist_path))?;

    // 이미 로드되어 있다면 먼저 언로드
    let _ = Command::new("launchctl")
        .args(["unload", &plist_path.to_string_lossy()])
        .output();

    // launchctl load
    let output = Command::new("launchctl")
        .args(["load", &plist_path.to_string_lossy()])
        .output()
        .context("launchctl load 실행 실패")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("launchctl load 실패: {}", stderr));
    }

    Ok(())
}

/// launchd 서비스 중지 및 제거
fn launchd_stop() -> Result<()> {
    let plist_path = launchd_plist_path()?;

    if !plist_path.exists() {
        return Ok(());
    }

    let output = Command::new("launchctl")
        .args(["unload", &plist_path.to_string_lossy()])
        .output()
        .context("launchctl unload 실행 실패")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("launchctl unload 실패: {}", stderr));
    }

    // plist 파일 삭제
    fs::remove_file(&plist_path)
        .with_context(|| format!("plist 파일 삭제 실패: {:?}", plist_path))?;

    Ok(())
}

/// launchd 서비스 상태 확인
fn launchd_status() -> Result<()> {
    let output = Command::new("launchctl")
        .args(["list", "com.stick.agent"])
        .output()
        .context("launchctl list 실행 실패")?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("✅ stick 서비스가 실행 중입니다.");
        println!("{}", stdout);
    } else {
        println!("⏹️  stick 서비스가 실행되고 있지 않습니다.");
    }
    Ok(())
}

// ── systemd (Linux) ──────────────────────────────────────────────────────

/// systemd unit 파일 경로
/// ~/.config/systemd/user/stick.service
fn systemd_unit_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("홈 디렉토리를 찾을 수 없습니다")?;
    Ok(home
        .join(".config")
        .join("systemd")
        .join("user")
        .join("stick.service"))
}

/// systemd unit 파일 내용 생성
fn generate_systemd_unit(stick_binary: &str) -> String {
    format!(
        r#"[Unit]
Description=stick - 한글 파일명 NFC 정규화 서비스
After=default.target

[Service]
Type=simple
ExecStart={binary} watch
Restart=on-failure
RestartSec=5
Environment=STICK_LOG=info

[Install]
WantedBy=default.target
"#,
        binary = stick_binary,
    )
}

/// systemd 서비스 등록 및 시작
fn systemd_start() -> Result<()> {
    let unit_path = systemd_unit_path()?;
    let stick_binary = find_stick_binary()?;

    if let Some(parent) = unit_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("systemd user 디렉토리 생성 실패: {:?}", parent))?;
    }

    let unit_content = generate_systemd_unit(&stick_binary.to_string_lossy());
    fs::write(&unit_path, &unit_content)
        .with_context(|| format!("unit 파일 생성 실패: {:?}", unit_path))?;

    // systemctl --user daemon-reload
    let reload = Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .output()
        .context("systemctl daemon-reload 실패")?;
    if !reload.status.success() {
        let stderr = String::from_utf8_lossy(&reload.stderr);
        return Err(anyhow::anyhow!("daemon-reload 실패: {}", stderr));
    }

    // systemctl --user enable --now stick.service
    let enable = Command::new("systemctl")
        .args(["--user", "enable", "--now", "stick.service"])
        .output()
        .context("systemctl enable 실패")?;
    if !enable.status.success() {
        let stderr = String::from_utf8_lossy(&enable.stderr);
        return Err(anyhow::anyhow!("서비스 활성화 실패: {}", stderr));
    }

    Ok(())
}

/// systemd 서비스 중지
fn systemd_stop() -> Result<()> {
    let output = Command::new("systemctl")
        .args(["--user", "stop", "stick.service"])
        .output()
        .context("systemctl stop 실패")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("서비스 중지 실패: {}", stderr));
    }

    // disable도 함께
    let _ = Command::new("systemctl")
        .args(["--user", "disable", "stick.service"])
        .output();

    // unit 파일 삭제
    let unit_path = systemd_unit_path()?;
    if unit_path.exists() {
        let _ = fs::remove_file(&unit_path);
    }

    let _ = Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .output();

    Ok(())
}

/// systemd 서비스 상태 확인
fn systemd_status() -> Result<()> {
    let output = Command::new("systemctl")
        .args(["--user", "status", "stick.service"])
        .output()
        .context("systemctl status 실패")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("{}", stdout);
    Ok(())
}

// ── 공개 API ─────────────────────────────────────────────────────────────

/// 데몬 서비스 시작 (OS 자동 감지)
pub fn start_service(log_dir: &str) -> Result<()> {
    match detect_service_manager()? {
        ServiceManager::Launchd => launchd_start(log_dir),
        ServiceManager::Systemd => systemd_start(),
    }
}

/// 데몬 서비스 중지
pub fn stop_service() -> Result<()> {
    match detect_service_manager()? {
        ServiceManager::Launchd => launchd_stop(),
        ServiceManager::Systemd => systemd_stop(),
    }
}

/// 데몬 서비스 상태 확인
pub fn show_status() -> Result<()> {
    match detect_service_manager()? {
        ServiceManager::Launchd => launchd_status(),
        ServiceManager::Systemd => systemd_status(),
    }
}
