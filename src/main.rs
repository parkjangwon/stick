// ============================================================================
// stick - macOS/Linux 한글 파일명 NFC 정규화 도구
// 진입점 (main.rs)
// ============================================================================

mod cli;
mod config;
mod daemon;
mod logger;
mod scanner;
mod tui;
mod watcher;
mod notifier;

use anyhow::Result;
use clap::Parser;
use std::io::{self, Write};

use cli::{Cli, Commands};
use config::StickConfig;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        // ── stick start ──────────────────────────────────────────────
        Commands::Start => {
            let config = StickConfig::load()?;

            // 설정 유효성 검증
            let warnings = config.validate();
            for warning in &warnings {
                eprintln!("⚠️  {}", warning);
            }

            if config.watch_paths.is_empty() {
                eprintln!("\n감시할 폴더가 없습니다.");
                eprintln!("먼저 'stick config' 명령으로 감시 폴더를 추가해주세요.");
                std::process::exit(1);
            }

            // 로그 경로 확인
            let log_dir = config.log_dir();
            let log_dir_str = log_dir.to_string_lossy().to_string();

            println!("🚀 stick 서비스를 시작합니다...");
            println!("   감시 폴더: {:?}", config.watch_paths);
            println!("   로그 경로: {}", log_dir_str);

            daemon::start_service(&log_dir_str)?;
        }

        // ── stick stop ───────────────────────────────────────────────
        Commands::Stop => {
            daemon::stop_service()?;
        }

        // ── stick status ─────────────────────────────────────────────
        Commands::Status => {
            daemon::show_status()?;
        }

        // ── stick config ─────────────────────────────────────────────
        Commands::Config => {
            tui::run_config_tui()?;
        }

        // ── stick scan ───────────────────────────────────────────────
        Commands::Scan { dry_run, yes } => {
            let config = StickConfig::load()?;

            // 로거 초기화 (콘솔 출력 포함)
            let log_dir = config.log_dir();
            logger::init_logger(&log_dir, true, &config.log_level)?;

            if config.watch_paths.is_empty() {
                eprintln!("감시할 폴더가 없습니다.");
                eprintln!("먼저 'stick config' 명령으로 감시 폴더를 추가해주세요.");
                std::process::exit(1);
            }

            // dry_run 모드 안내
            if dry_run {
                println!("🔍 미리보기 모드 (실제 변경 없음)");
                println!("─────────────────────────────────");
            } else {
                println!("🔄 스캔 및 변환 모드");
                println!("─────────────────────────────────");
            }

            // 대화형 확인 (--yes 플래그 없고, config에서 활성화된 경우)
            let should_confirm = !yes && !dry_run && config.confirm_before_scan;

            // 먼저 미리보기 실행
            if should_confirm {
                println!("\n📋 변환 대상 미리보기:");
                let mut total_targets = 0;

                for watch_path in &config.watch_paths {
                    let path = std::path::Path::new(watch_path);
                    if !path.exists() {
                        eprintln!("⚠️  경로 없음: {}", watch_path);
                        continue;
                    }
                    match scanner::scan_directory(path, &config, true) {
                        Ok(result) => {
                            total_targets += result.renamed.len();
                            for entry in &result.renamed {
                                println!(
                                    "  {} → {}",
                                    entry.original_name, entry.new_name
                                );
                            }
                        }
                        Err(err) => {
                            eprintln!("❌ 스캔 에러 ({}): {}", watch_path, err);
                        }
                    }
                }

                if total_targets == 0 {
                    println!("\n✅ 변환이 필요한 파일이 없습니다.");
                    return Ok(());
                }

                // 사용자 확인
                print!("\n총 {}개 파일을 변환하시겠습니까? [y/N] ", total_targets);
                io::stdout().flush()?;

                let mut answer = String::new();
                io::stdin().read_line(&mut answer)?;

                if !answer.trim().eq_ignore_ascii_case("y") {
                    println!("취소되었습니다.");
                    return Ok(());
                }
            }

            // 실제 스캔 실행
            let mut total_renamed = 0;
            let mut total_errors = 0;

            for watch_path in &config.watch_paths {
                let path = std::path::Path::new(watch_path);
                if !path.exists() {
                    eprintln!("⚠️  경로 없음: {}", watch_path);
                    continue;
                }

                println!("\n📂 스캔: {}", watch_path);

                match scanner::scan_directory(path, &config, dry_run) {
                    Ok(result) => {
                        total_renamed += result.renamed.len();
                        total_errors += result.errors.len();

                        for entry in &result.renamed {
                            let action = if dry_run { "변환 예정" } else { "변환 완료" };
                            println!(
                                "  ✅ [{}] {} → {}",
                                action, entry.original_name, entry.new_name
                            );
                        }
                        for error in &result.errors {
                            println!("  ❌ {}", error);
                        }
                    }
                    Err(err) => {
                        eprintln!("❌ 스캔 에러 ({}): {}", watch_path, err);
                        total_errors += 1;
                    }
                }
            }

            // 최종 요약
            println!("\n═══════════════════════════════════");
            let action_word = if dry_run { "변환 예정" } else { "변환 완료" };
            println!(
                "📊 {} {}건, 에러 {}건",
                action_word, total_renamed, total_errors
            );
        }

        // ── stick watch (내부 명령: 데몬에서 호출) ────────────────────
        Commands::Watch => {
            let config = StickConfig::load()?;
            let log_dir = config.log_dir();

            // 데몬 모드 → 콘솔 출력 없이 파일만 기록
            logger::init_logger(&log_dir, false, &config.log_level)?;

            tracing::debug!("stick 감시 데몬 시작");
            watcher::run_watch_loop(&config)?;
        }
    }

    Ok(())
}
