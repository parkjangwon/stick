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
