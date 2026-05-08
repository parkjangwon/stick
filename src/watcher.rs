// ============================================================================
// stick - 파일시스템 감시 모듈 (하이브리드 방식)
// 실시간 이벤트(notify) + 주기적 전체 스캔 병행
// ============================================================================

use anyhow::{Context, Result};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;
use tracing::{debug, error, warn};

use crate::config::StickConfig;
use crate::scanner;

/// 감시 모드 실행 (데몬에서 호출)
///
/// 하이브리드 방식:
/// 1. notify로 파일 생성/이름변경 이벤트를 실시간 감지
/// 2. scan_interval_seconds마다 전체 스캔으로 놓친 이벤트 보완
pub fn run_watch_loop(config: &StickConfig) -> Result<()> {
    debug!("🔍 stick 감시 모드 시작");
    debug!("감시 대상 폴더: {:?}", config.watch_paths);
    debug!(
        "전체 스캔 간격: {}초",
        config.scan_interval_seconds
    );

    // 이벤트 채널 생성
    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();

    // 파일시스템 감시자 생성
    let mut watcher = RecommendedWatcher::new(tx, Config::default())
        .context("파일시스템 감시자 생성 실패")?;

    // 각 감시 경로 등록
    let watch_mode = if config.recursive {
        RecursiveMode::Recursive
    } else {
        RecursiveMode::NonRecursive
    };

    for watch_path in &config.watch_paths {
        let path = Path::new(watch_path);
        if !path.exists() {
            warn!("감시 경로가 존재하지 않아 건너뜁니다: {}", watch_path);
            continue;
        }
        watcher
            .watch(path, watch_mode)
            .with_context(|| format!("감시 등록 실패: {}", watch_path))?;
        debug!("📂 감시 등록: {}", watch_path);
    }

    // 시작 시 1회 전체 스캔 수행
    debug!("🔄 초기 전체 스캔 시작...");
    run_full_scan(config);

    // 이벤트 루프 (타임아웃 기반 하이브리드)
    let scan_interval = Duration::from_secs(config.scan_interval_seconds);

    loop {
        // 타임아웃까지 이벤트 대기 → 타임아웃되면 전체 스캔 실행
        match rx.recv_timeout(scan_interval) {
            Ok(Ok(event)) => {
                handle_fs_event(&event, config);
            }
            Ok(Err(watch_error)) => {
                error!("파일 감시 에러: {:?}", watch_error);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // 주기적 전체 스캔 (놓친 이벤트 보완)
                debug!("🔄 주기적 전체 스캔 실행...");
                run_full_scan(config);
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                // 감시자가 종료됨 → 루프 탈출
                warn!("파일 감시 채널이 닫혔습니다. 감시를 종료합니다.");
                break;
            }
        }
    }

    Ok(())
}

/// 파일시스템 이벤트 처리
/// Create, Rename 이벤트만 NFD→NFC 변환 대상으로 처리합니다.
fn handle_fs_event(event: &Event, config: &StickConfig) {
    // 파일 생성 또는 이름 변경 이벤트만 관심 대상
    let is_relevant = matches!(
        event.kind,
        EventKind::Create(_) | EventKind::Modify(notify::event::ModifyKind::Name(_))
    );

    if !is_relevant {
        return;
    }

    // 이벤트에 포함된 각 경로에 대해 NFC 변환 시도
    for path in &event.paths {
        if !path.exists() {
            continue; // 이미 삭제되었거나 이동된 파일
        }

        match scanner::normalize_single_path(path, config) {
            Ok(Some(entry)) => {
                debug!(
                    "[실시간] {} → {} (NFD→NFC)",
                    entry.original_name, entry.new_name
                );
            }
            Ok(None) => {
                // 변환 불필요 또는 제외 대상 → 무시
            }
            Err(err) => {
                error!("[실시간 에러] {}: {}", path.display(), err);
            }
        }
    }
}

/// 전체 스캔 실행 (주기적 보완용)
fn run_full_scan(config: &StickConfig) {
    for watch_path in &config.watch_paths {
        let path = Path::new(watch_path);
        if !path.exists() {
            continue;
        }
        match scanner::scan_directory(path, config, false) {
            Ok(result) => {
                if !result.renamed.is_empty() {
                    debug!(
                        "[전체스캔] {} - 변환 {}건",
                        watch_path,
                        result.renamed.len()
                    );
                }
            }
            Err(err) => {
                error!("[전체스캔 에러] {}: {}", watch_path, err);
            }
        }
    }
}
