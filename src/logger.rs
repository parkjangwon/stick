// ============================================================================
// stick - 로그 관리 모듈
// 로그 경로: ~/logs/stick/stick_YYYY-MM-DD.log
// ============================================================================

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// 로깅 시스템 초기화
///
/// 1. 콘솔 출력 (stderr)
/// 2. 파일 출력 - 일별 로테이션
pub fn init_logger(log_dir: &Path, console_output: bool) -> Result<()> {
    fs::create_dir_all(log_dir)
        .with_context(|| format!("로그 디렉토리 생성 실패: {:?}", log_dir))?;

    let file_appender = RollingFileAppender::new(Rotation::DAILY, log_dir, "stick.log");
    let env_filter = EnvFilter::try_from_env("STICK_LOG")
        .unwrap_or_else(|_| EnvFilter::new("info"));

    if console_output {
        let file_layer = fmt::layer()
            .with_writer(file_appender)
            .with_ansi(false)
            .with_target(false);
        let console_layer = fmt::layer()
            .with_writer(std::io::stderr)
            .with_target(false);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(file_layer)
            .with(console_layer)
            .init();
    } else {
        let file_layer = fmt::layer()
            .with_writer(file_appender)
            .with_ansi(false)
            .with_target(false);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(file_layer)
            .init();
    }

    Ok(())
}
