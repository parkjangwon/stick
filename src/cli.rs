// ============================================================================
// stick - macOS/Linux 한글 파일명 NFC 정규화 도구
// CLI 명령어 정의 모듈
// ============================================================================

use clap::{Parser, Subcommand};

/// stick - macOS/Linux 한글 파일명 NFC 정규화 도구
///
/// macOS에서 한글 파일명이 NFD(자소 분리)로 저장되어
/// Windows에서 깨져 보이는 문제를 해결합니다.
#[derive(Parser, Debug)]
#[command(
    name = "stick",
    version,
    about = "한글 파일명 NFC 정규화 도구 🇰🇷",
    long_about = "macOS/Linux에서 한글 파일명이 NFD(자소 분리)로 저장되어\n\
                  Windows에서 깨져 보이는 문제를 해결합니다.\n\
                  파일명을 NFC(자소 결합) 형태로 변환하여\n\
                  크로스 플랫폼 호환성을 보장합니다."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 데몬 모드로 백그라운드 실행 (폴더 감시 시작)
    Start,

    /// 실행 중인 데몬 중지
    Stop,

    /// 데몬 실행 상태 확인
    Status,

    /// TUI 설정 메뉴 열기
    Tui,

    /// 현재 설치된 stick 버전 정보 출력
    Version,

    /// 일회성 스캔 및 변환
    Scan {
        /// 실제 변경 없이 미리보기만 표시
        #[arg(long, help = "실제 변경 없이 변환 대상만 미리보기")]
        dry_run: bool,

        /// 확인 없이 바로 실행
        #[arg(short, long, help = "대화형 확인 없이 바로 변환 실행")]
        yes: bool,
    },

    /// 감시 모드 실행 (데몬 서비스가 내부적으로 호출)
    #[command(hide = true)]
    Watch,
}
