// ============================================================================
// stick - 시스템 데스크톱 알림 연동 모듈
// macOS: osascript를 통한 네이티브 알림 발송
// ============================================================================

#[cfg(target_os = "macos")]
use std::process::Command;

/// macOS 시스템 알림을 보냅니다.
/// osascript의 'display notification' 명령어를 사용합니다.
#[cfg(target_os = "macos")]
pub fn send_notification(title: &str, message: &str) {
    // 따옴표 문자 이스케이프 처리하여 셸 에러 방지
    let safe_message = message.replace('"', "\\\"");
    let safe_title = title.replace('"', "\\\"");

    // osascript 명령어 작성 (사운드 효과 Glass 추가)
    let script = format!(
        "display notification \"{}\" with title \"{}\" sound name \"Glass\"",
        safe_message, safe_title
    );

    let _ = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .status();
}

/// macOS가 아닌 환경에서는 작동하지 않도록 빈 함수 정의
#[cfg(not(target_os = "macos"))]
pub fn send_notification(_title: &str, _message: &str) {}
