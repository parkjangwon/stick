// ============================================================================
// stick - 로그 관리 모듈
// 로그 경로: ~/logs/stick/stick_YYYY-MM-DD.log
// ============================================================================

use anyhow::{Context, Result};
use chrono::Local;
use std::fs;
use std::path::Path;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, fmt::format::Writer, fmt::time::FormatTime, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// 로컬 시간으로 포맷팅하기 위한 커스텀 타이머
struct LocalTimer;

impl FormatTime for LocalTimer {
    fn format_time(&self, w: &mut Writer<'_>) -> std::fmt::Result {
        write!(w, "{}", Local::now().format("%Y-%m-%dT%H:%M:%S%.3f%z"))
    }
}

/// 로깅 시스템 초기화
///
/// 1. 콘솔 출력 (stderr)
/// 2. 파일 출력 - 일별 로테이션
pub fn init_logger(log_dir: &Path, console_output: bool, level: &str) -> Result<()> {
    fs::create_dir_all(log_dir)
        .with_context(|| format!("로그 디렉토리 생성 실패: {:?}", log_dir))?;

    let file_appender = RollingFileAppender::new(Rotation::DAILY, log_dir, "stick.log");
    let env_filter = EnvFilter::try_from_env("STICK_LOG")
        .unwrap_or_else(|_| EnvFilter::new(level));

    if console_output {
        let file_layer = fmt::layer()
            .with_writer(file_appender)
            .with_ansi(false)
            .with_target(false)
            .with_timer(LocalTimer);
        let console_layer = fmt::layer()
            .with_writer(std::io::stderr)
            .with_target(false)
            .with_timer(LocalTimer);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(file_layer)
            .with(console_layer)
            .init();
    } else {
        let file_layer = fmt::layer()
            .with_writer(file_appender)
            .with_ansi(false)
            .with_target(false)
            .with_timer(LocalTimer);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(file_layer)
            .init();
    }

    Ok(())
}

/// 로그 디렉토리의 총 크기를 바이트 단위로 계산합니다.
pub fn get_dir_size(path: &Path) -> Result<u64> {
    let mut size = 0;
    if path.exists() && path.is_dir() {
        for entry in walkdir::WalkDir::new(path).min_depth(1) {
            let entry = entry?;
            if entry.file_type().is_file() {
                size += entry.metadata()?.len();
            }
        }
    }
    Ok(size)
}

/// 오늘 날짜 이전의 .log 파일들을 찾아 gzip으로 압축하고 원본을 삭제합니다.
pub fn compress_old_logs(log_dir: &Path) -> Result<()> {
    if !log_dir.exists() {
        return Ok(());
    }

    let today = Local::now().format("%Y-%m-%d").to_string();
    let today_log = format!("stick.log.{}", today);

    for entry in fs::read_dir(log_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if !path.is_file() {
            continue;
        }

        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            // "stick.log.YYYY-MM-DD" 형식이면서 오늘의 로그가 아닌 경우
            if name.starts_with("stick.log.") && !name.ends_with(".gz") && name != "stick.log" && name != today_log {
                let out_path = format!("{}.gz", path.display());
                
                // 파일 압축 진행
                if let Ok(file_in) = fs::File::open(&path) {
                    if let Ok(file_out) = fs::File::create(&out_path) {
                        let mut input = std::io::BufReader::new(file_in);
                        let mut output = flate2::write::GzEncoder::new(file_out, flate2::Compression::default());
                        
                        if std::io::copy(&mut input, &mut output).is_ok() && output.finish().is_ok() {
                            // 압축 성공 시 원본 삭제
                            let _ = fs::remove_file(&path);
                            tracing::debug!("로그 압축 완료: {} -> {}", name, out_path);
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}
