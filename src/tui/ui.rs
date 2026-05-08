// ============================================================================
// stick - TUI 화면 렌더링 모듈
// ratatui 위젯으로 한글 설정 인터페이스 구성
// ============================================================================

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

use super::app::{App, Screen};

/// 메인 렌더링 함수 - 현재 화면에 맞는 UI 그리기
pub fn render(frame: &mut Frame, app: &App) {
    // 배경을 깔끔하게 지웁니다
    frame.render_widget(Clear, frame.area());

    // 미니멀한 중앙 레이아웃 계산 (최대 너비 70, 최대 높이 24 제한)
    let term_area = frame.area();
    let width = std::cmp::min(70, term_area.width);
    let height = std::cmp::min(24, term_area.height);
    
    let x = term_area.x + (term_area.width.saturating_sub(width)) / 2;
    let y = term_area.y + (term_area.height.saturating_sub(height)) / 2;
    
    let app_area = Rect::new(x, y, width, height);

    // 팝업 창처럼 보이게 하기 위해 배경색 설정 가능하지만, 미니멀을 위해 패스
    // 전체 레이아웃: 헤더 + 콘텐츠 + 상태바
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // 헤더
            Constraint::Min(10),   // 콘텐츠
            Constraint::Length(3), // 상태바
        ])
        .split(app_area);

    // 헤더
    render_header(frame, chunks[0], app);

    // 콘텐츠 (화면별 분기)
    match app.current_screen {
        Screen::Main => render_main_menu(frame, chunks[1], app),
        Screen::WatchPaths => render_watch_paths(frame, chunks[1], app),
        Screen::ExcludeSettings => render_exclude_settings(frame, chunks[1], app),
        Screen::ExcludeExtensions => render_list_editor(
            frame,
            chunks[1],
            app,
            "제외 확장자",
            &app.config.exclude_extensions,
        ),
        Screen::ExcludeDirs => render_list_editor(
            frame,
            chunks[1],
            app,
            "제외 폴더",
            &app.config.exclude_directories,
        ),
        Screen::LogSettings => render_log_settings(frame, chunks[1], app),
        Screen::GeneralSettings => render_general_settings(frame, chunks[1], app),
        Screen::DirPicker => render_dir_picker(frame, chunks[1], app),
    }

    // 상태바
    render_status_bar(frame, chunks[2], app);

    // 텍스트 입력 모달 (입력 모드일 때 오버레이)
    if app.input_mode {
        render_input_modal(frame, app);
    }

    // 삭제 확인 모달
    if app.confirm_delete {
        render_confirm_modal(frame);
    }
}

/// 헤더 렌더링
fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let title = match app.current_screen {
        Screen::Main => " 🔧 stick 설정 ",
        Screen::WatchPaths => " 📂 감시 폴더 관리 ",
        Screen::ExcludeSettings => " 🚫 제외 설정 ",
        Screen::ExcludeExtensions => " 📄 제외 확장자 ",
        Screen::ExcludeDirs => " 📁 제외 폴더 ",
        Screen::LogSettings => " 📋 로그 설정 ",
        Screen::GeneralSettings => " ⚙️  일반 설정 ",
        Screen::DirPicker => " 🔍 폴더 선택기 ",
    };

    let header = Paragraph::new(title)
        .style(Style::default().fg(Color::Cyan).bold())
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );

    frame.render_widget(header, area);
}

/// 메인 메뉴 렌더링
fn render_main_menu(frame: &mut Frame, area: Rect, app: &App) {
    let items = vec![
        format!("📂 감시 폴더 관리 ({}개)", app.config.watch_paths.len()),
        "🚫 제외 설정".to_string(),
        "📋 로그 설정".to_string(),
        "⚙️  일반 설정".to_string(),
        "💾 저장 및 종료".to_string(),
        "❌ 취소".to_string(),
    ];

    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == app.selected_index {
                Style::default().fg(Color::Yellow).bold()
            } else {
                Style::default().fg(Color::White)
            };
            let prefix = if i == app.selected_index { "▶ " } else { "  " };
            ListItem::new(format!("{}{}", prefix, item)).style(style)
        })
        .collect();

    let list = List::new(list_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" 메뉴 ")
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(list, area);
}

/// 감시 폴더 목록 렌더링
fn render_watch_paths(frame: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = if app.config.watch_paths.is_empty() {
        vec![ListItem::new("  (등록된 폴더 없음 - [a]로 추가)")
            .style(Style::default().fg(Color::DarkGray).italic())]
    } else {
        app.config
            .watch_paths
            .iter()
            .enumerate()
            .map(|(i, path)| {
                let style = if i == app.selected_index {
                    Style::default().fg(Color::Yellow).bold()
                } else {
                    Style::default().fg(Color::White)
                };
                let prefix = if i == app.selected_index { "▶ " } else { "  " };
                // 경로 존재 여부 표시
                let exists = std::path::Path::new(path).exists();
                let indicator = if exists { "✅" } else { "⚠️ " };
                ListItem::new(format!("{}{} {}", prefix, indicator, path)).style(style)
            })
            .collect()
    };

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" 감시 폴더 [a]추가 [d]삭제 [Esc]뒤로 ")
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(list, area);
}

/// 제외 설정 메뉴 렌더링
fn render_exclude_settings(frame: &mut Frame, area: Rect, app: &App) {
    let toggle = |v: bool| if v { "✅" } else { "⬜" };

    let items = vec![
        format!(
            "{} 숨김 파일 제외 (.으로 시작하는 파일)",
            toggle(app.config.exclude_hidden)
        ),
        format!(
            "{} 심볼릭 링크 제외",
            toggle(app.config.exclude_symlinks)
        ),
        format!(
            "{} 임시 파일 제외 (~로 끝나는 파일)",
            toggle(app.config.exclude_temp_files)
        ),
        format!(
            "📄 제외 확장자 관리 ({}개)",
            app.config.exclude_extensions.len()
        ),
        format!(
            "📁 제외 폴더 관리 ({}개)",
            app.config.exclude_directories.len()
        ),
        format!(
            "{} 스캔 시 대화형 확인",
            toggle(app.config.confirm_before_scan)
        ),
    ];

    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == app.selected_index {
                Style::default().fg(Color::Yellow).bold()
            } else {
                Style::default().fg(Color::White)
            };
            let prefix = if i == app.selected_index { "▶ " } else { "  " };
            ListItem::new(format!("{}{}", prefix, item)).style(style)
        })
        .collect();

    let list = List::new(list_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" 제외 설정 [Space/Enter]토글 [Esc]뒤로 ")
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(list, area);
}

/// 리스트 편집기 (확장자, 폴더명 공용)
fn render_list_editor(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    title: &str,
    items_data: &[String],
) {
    let items: Vec<ListItem> = if items_data.is_empty() {
        vec![ListItem::new(format!("  (등록된 {} 없음 - [a]로 추가)", title))
            .style(Style::default().fg(Color::DarkGray).italic())]
    } else {
        items_data
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let style = if i == app.selected_index {
                    Style::default().fg(Color::Yellow).bold()
                } else {
                    Style::default().fg(Color::White)
                };
                let prefix = if i == app.selected_index { "▶ " } else { "  " };
                ListItem::new(format!("{}{}", prefix, item)).style(style)
            })
            .collect()
    };

    let block_title = format!(" {} [a]추가 [d]삭제 [Esc]뒤로 ", title);
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(block_title)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(list, area);
}

/// 로그 설정 렌더링
fn render_log_settings(frame: &mut Frame, area: Rect, app: &App) {
    let items = vec![
        format!("📂 로그 경로: {}", app.config.log_path),
        format!("🎚️  로그 레벨: {}", app.config.log_level),
        "📊 로그 디렉토리 크기 확인".to_string(),
    ];

    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == app.selected_index {
                Style::default().fg(Color::Yellow).bold()
            } else {
                Style::default().fg(Color::White)
            };
            let prefix = if i == app.selected_index { "▶ " } else { "  " };
            ListItem::new(format!("{}{}", prefix, item)).style(style)
        })
        .collect();

    let list = List::new(list_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" 로그 설정 [Enter]편집 [Esc]뒤로 ")
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(list, area);
}

/// 일반 설정 렌더링
fn render_general_settings(frame: &mut Frame, area: Rect, app: &App) {
    let toggle = |v: bool| if v { "✅" } else { "⬜" };

    let items = vec![
        format!(
            "{} 하위 폴더 재귀 탐색",
            toggle(app.config.recursive)
        ),
        format!(
            "⏱️  감시 스캔 간격: {}초",
            app.config.scan_interval_seconds
        ),
        format!(
            "{} 스캔 시 대화형 확인",
            toggle(app.config.confirm_before_scan)
        ),
    ];

    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == app.selected_index {
                Style::default().fg(Color::Yellow).bold()
            } else {
                Style::default().fg(Color::White)
            };
            let prefix = if i == app.selected_index { "▶ " } else { "  " };
            ListItem::new(format!("{}{}", prefix, item)).style(style)
        })
        .collect();

    let list = List::new(list_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" 일반 설정 [Space/Enter]토글 [Esc]뒤로 ")
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(list, area);
}

/// 상태바 렌더링
fn render_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let status = Paragraph::new(app.status_message.as_str())
        .style(Style::default().fg(Color::Green))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

    frame.render_widget(status, area);
}

/// 텍스트 입력 모달 오버레이
fn render_input_modal(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 20, frame.area());

    // 배경 클리어
    frame.render_widget(Clear, area);

    let label = match &app.input_target {
        Some(super::app::InputTarget::AddExcludeExtension) => "제외 확장자 입력 (.ext):",
        Some(super::app::InputTarget::AddExcludeDir) => "제외 폴더명 입력:",
        Some(super::app::InputTarget::EditLogPath) => "로그 경로 입력:",
        Some(super::app::InputTarget::EditScanInterval) => "스캔 간격(초) 입력:",
        None => "입력:",
    };

    let input_text = format!("{}\n\n> {}▏", label, app.input_buffer);

    let input = Paragraph::new(input_text)
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" ✏️  입력 [Enter]확인 [Esc]취소 ")
                .border_style(Style::default().fg(Color::Cyan)),
        );

    frame.render_widget(input, area);
}

/// 삭제 확인 모달
fn render_confirm_modal(frame: &mut Frame) {
    let area = centered_rect(40, 15, frame.area());
    frame.render_widget(Clear, area);

    let text = "정말 삭제하시겠습니까?\n\n  [y] 예  [n] 아니오";
    let confirm = Paragraph::new(text)
        .style(Style::default().fg(Color::Red))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" ⚠️  삭제 확인 ")
                .border_style(Style::default().fg(Color::Red)),
        );

    frame.render_widget(confirm, area);
}

/// 화면 중앙에 위치한 Rect 계산 (모달용)
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// 디렉토리 탐색기 렌더링
fn render_dir_picker(frame: &mut Frame, area: Rect, app: &App) {
    let dp = match &app.dir_picker {
        Some(dp) => dp,
        None => return,
    };

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // 현재 경로
            Constraint::Min(5),    // 리스트
            Constraint::Length(3), // 버튼들
        ])
        .split(area);

    // 1. 현재 경로
    let current_path = Paragraph::new(format!(" 현재 위치: {}", dp.current_dir.display()))
        .style(Style::default().fg(Color::Cyan).bold());
    frame.render_widget(current_path, layout[0]);

    // 2. 디렉토리 리스트
    let items: Vec<ListItem> = dp.items.iter().enumerate().map(|(i, path)| {
        let is_selected = dp.selected_index == i;
        let is_checked = dp.selected_paths.contains(path);
        
        let prefix = if is_selected { "▶ " } else { "  " };
        let check = if is_checked { "[x]" } else { "[ ]" };
        
        let name = if path.file_name().map(|n| n == "..").unwrap_or(false) || path == &dp.current_dir.parent().unwrap_or(std::path::Path::new("")) {
            "📁 .. (상위 폴더로)".to_string()
        } else {
            format!("📁 {}", path.file_name().unwrap_or_default().to_string_lossy())
        };

        let style = if is_selected && dp.focus == super::app::DirPickerFocus::List {
            Style::default().fg(Color::Yellow).bold()
        } else if is_checked {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::White)
        };

        ListItem::new(format!("{}{} {}", prefix, check, name)).style(style)
    }).collect();

    let list_style = if dp.focus == super::app::DirPickerFocus::List {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(list_style),
    );
    frame.render_widget(list, layout[1]);

    // 3. 확인 / 취소 버튼
    let btn_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(layout[2]);

    let confirm_style = if dp.focus == super::app::DirPickerFocus::Confirm {
        Style::default().fg(Color::Yellow).bold()
    } else {
        Style::default().fg(Color::White)
    };
    let confirm_btn = Paragraph::new(format!("✅ 선택 완료 ({}개)", dp.selected_paths.len()))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(confirm_style));
    frame.render_widget(confirm_btn, btn_layout[0]);

    let cancel_style = if dp.focus == super::app::DirPickerFocus::Cancel {
        Style::default().fg(Color::Yellow).bold()
    } else {
        Style::default().fg(Color::White)
    };
    let cancel_btn = Paragraph::new("❌ 취소")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(cancel_style));
    frame.render_widget(cancel_btn, btn_layout[1]);
}
