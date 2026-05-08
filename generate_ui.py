import sys

new_ui = """// ============================================================================
// stick - TUI 화면 렌더링 모듈
// ratatui 위젯으로 모던한 터미널 인터페이스 구성
// ============================================================================

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

use super::app::{App, Screen};

/// 메인 렌더링 함수 - 현재 화면에 맞는 UI 그리기
pub fn render(frame: &mut Frame, app: &mut App) {
    frame.render_widget(Clear, frame.area());

    let term_area = frame.area();
    let width = std::cmp::min(76, term_area.width);
    let height = std::cmp::min(26, term_area.height);
    
    let x = term_area.x + (term_area.width.saturating_sub(width)) / 2;
    let y = term_area.y + (term_area.height.saturating_sub(height)) / 2;
    
    let app_area = Rect::new(x, y, width, height);

    // 전체 레이아웃: 여백 + 헤더 + 여백 + 콘텐츠
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Top margin
            Constraint::Length(3),  // Header
            Constraint::Length(1),  // Spacing
            Constraint::Min(10),    // Content
        ])
        .split(app_area);

    // 헤더
    render_header(frame, chunks[1], app);

    // 콘텐츠 영역
    match app.current_screen {
        Screen::Main => render_menu_section(frame, chunks[3], app, "Choose an action", get_main_menu_items(app)),
        Screen::WatchPaths => render_watch_paths(frame, chunks[3], app),
        Screen::ExcludeSettings => render_menu_section(frame, chunks[3], app, "Exclude Settings", get_exclude_menu_items(app)),
        Screen::ExcludeExtensions => render_list_editor(frame, chunks[3], app, "Exclude Extensions", &app.config.exclude_extensions),
        Screen::ExcludeDirs => render_list_editor(frame, chunks[3], app, "Exclude Directories", &app.config.exclude_directories),
        Screen::LogSettings => render_menu_section(frame, chunks[3], app, "Log Settings", get_log_menu_items(app)),
        Screen::GeneralSettings => render_menu_section(frame, chunks[3], app, "General Settings", get_general_menu_items(app)),
        Screen::DirPicker => render_dir_picker(frame, chunks[3], app),
    }

    // 텍스트 입력 모달
    if app.input_mode && app.input_target != Some(super::app::InputTarget::DirPickerSearch) {
        render_input_modal(frame, app);
    }

    // 삭제 확인 모달
    if app.confirm_delete {
        render_confirm_modal(frame);
    }
}

fn render_header(frame: &mut Frame, area: Rect, _app: &App) {
    let title_line = Line::from(vec![
        Span::styled("STICK", Style::default().fg(Color::Cyan).bold()),
        Span::styled(" CLI", Style::default().fg(Color::DarkGray)),
        Span::styled("   •   ", Style::default().fg(Color::DarkGray)),
        Span::styled("Configuration Manager", Style::default().fg(Color::White).bold()),
    ]);

    let header = Paragraph::new(title_line)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .padding(ratatui::widgets::Padding::new(2, 2, 0, 0)),
        );

    frame.render_widget(header, area);
}

fn render_menu_section(frame: &mut Frame, area: Rect, app: &App, title: &str, items: Vec<String>) {
    let block = Block::default()
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(Color::DarkGray));
    
    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title
            Constraint::Length(2), // Subtitle + spacing
            Constraint::Min(0),    // List
        ])
        .split(inner_area);

    let title_para = Paragraph::new(format!(" {}", title))
        .style(Style::default().fg(Color::LightBlue).bold());
    frame.render_widget(title_para, layout[0]);

    let subtitle_para = Paragraph::new(format!(" {}", app.status_message))
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(subtitle_para, layout[1]);

    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == app.selected_index {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::White)
            };
            let prefix = if i == app.selected_index { "> " } else { "  " };
            ListItem::new(format!(" {}{}", prefix, item)).style(style)
        })
        .collect();

    let list = List::new(list_items);
    frame.render_widget(list, layout[2]);
}

fn get_main_menu_items(app: &App) -> Vec<String> {
    vec![
        format!("감시 폴더 관리 ({}개)", app.config.watch_paths.len()),
        "제외 설정".to_string(),
        "로그 설정".to_string(),
        "일반 설정".to_string(),
    ]
}

fn get_exclude_menu_items(app: &App) -> Vec<String> {
    let toggle = |v: bool| if v { "[x]" } else { "[ ]" };
    vec![
        format!("{} 숨김 파일 제외 (.으로 시작하는 파일)", toggle(app.config.exclude_hidden)),
        format!("{} 심볼릭 링크 제외", toggle(app.config.exclude_symlinks)),
        format!("{} 임시 파일 제외 (~로 끝나는 파일)", toggle(app.config.exclude_temp_files)),
        format!("   제외 확장자 관리 ({}개)", app.config.exclude_extensions.len()),
        format!("   제외 폴더 관리 ({}개)", app.config.exclude_directories.len()),
        format!("{} 스캔 시 대화형 확인", toggle(app.config.confirm_before_scan)),
    ]
}

fn get_log_menu_items(app: &App) -> Vec<String> {
    vec![
        format!("로그 경로: {}", app.config.log_path),
        format!("로그 레벨: {}", app.config.log_level),
        "로그 디렉토리 크기 확인".to_string(),
    ]
}

fn get_general_menu_items(app: &App) -> Vec<String> {
    let toggle = |v: bool| if v { "[x]" } else { "[ ]" };
    vec![
        format!("{} 하위 폴더 재귀 탐색", toggle(app.config.recursive)),
        format!("   감시 스캔 간격: {}초", app.config.scan_interval_seconds),
        format!("   변환 트리거 대기 시간: {}초", app.config.debounce_delay_seconds),
        format!("{} macOS 시스템 알림", toggle(app.config.enable_notifications)),
        format!("{} 부팅 시 자동 실행 (LaunchAgent)", toggle(app.config.auto_start)),
        format!("{} 스캔 시 대화형 확인", toggle(app.config.confirm_before_scan)),
    ]
}

fn render_watch_paths(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(Color::DarkGray));
    
    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Min(0),
        ])
        .split(inner_area);

    frame.render_widget(Paragraph::new(" 감시 폴더 리스트").style(Style::default().fg(Color::LightBlue).bold()), layout[0]);
    
    let mut subtitle = app.status_message.clone();
    if subtitle == "↑/↓ 이동  •  Enter 선택  •  Esc 취소" {
         subtitle = "a: 추가  •  d: 삭제  •  Esc: 뒤로".to_string();
    }
    frame.render_widget(Paragraph::new(format!(" {}", subtitle)).style(Style::default().fg(Color::DarkGray)), layout[1]);

    let items: Vec<ListItem> = if app.config.watch_paths.is_empty() {
        vec![ListItem::new("    (등록된 폴더 없음)").style(Style::default().fg(Color::DarkGray).italic())]
    } else {
        app.config.watch_paths.iter().enumerate().map(|(i, path)| {
            let style = if i == app.selected_index { Style::default().fg(Color::Green) } else { Style::default().fg(Color::White) };
            let prefix = if i == app.selected_index { "> " } else { "  " };
            let exists = std::path::Path::new(path).exists();
            let indicator = if exists { " " } else { "!" };
            ListItem::new(format!(" {}{} {}", prefix, indicator, path)).style(style)
        }).collect()
    };

    frame.render_widget(List::new(items), layout[2]);
}

fn render_list_editor(frame: &mut Frame, area: Rect, app: &App, title: &str, items_data: &[String]) {
    let block = Block::default()
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(Color::DarkGray));
    
    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Min(0),
        ])
        .split(inner_area);

    frame.render_widget(Paragraph::new(format!(" {}", title)).style(Style::default().fg(Color::LightBlue).bold()), layout[0]);
    
    let mut subtitle = app.status_message.clone();
    if subtitle == "↑/↓ 이동  •  Enter 선택  •  Esc 취소" {
         subtitle = "a: 추가  •  d: 삭제  •  Esc: 뒤로".to_string();
    }
    frame.render_widget(Paragraph::new(format!(" {}", subtitle)).style(Style::default().fg(Color::DarkGray)), layout[1]);

    let items: Vec<ListItem> = if items_data.is_empty() {
        vec![ListItem::new("    (등록된 항목 없음)").style(Style::default().fg(Color::DarkGray).italic())]
    } else {
        items_data.iter().enumerate().map(|(i, item)| {
            let style = if i == app.selected_index { Style::default().fg(Color::Green) } else { Style::default().fg(Color::White) };
            let prefix = if i == app.selected_index { "> " } else { "  " };
            ListItem::new(format!(" {}{}", prefix, item)).style(style)
        }).collect()
    };

    frame.render_widget(List::new(items), layout[2]);
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let w = std::cmp::min(width, area.width);
    let h = std::cmp::min(height, area.height);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    Rect::new(x, y, w, h)
}

fn render_input_modal(frame: &mut Frame, app: &App) {
    let area = centered_rect(50, 5, frame.area());
    frame.render_widget(Clear, area);

    let label = match &app.input_target {
        Some(super::app::InputTarget::AddExcludeExtension) => "제외 확장자 입력 (.ext):",
        Some(super::app::InputTarget::AddExcludeDir) => "제외 폴더명 입력:",
        Some(super::app::InputTarget::EditLogPath) => "로그 경로 입력:",
        Some(super::app::InputTarget::EditScanInterval) => "스캔 간격(초) 입력:",
        Some(super::app::InputTarget::EditDebounceDelay) => "대기 시간(초) 입력:",
        Some(super::app::InputTarget::DirPickerSearch) => "폴더 검색:",
        None => "입력:",
    };

    let input_text = format!("{}\\n\\n> {}▏", label, app.input_buffer);

    let input = Paragraph::new(input_text)
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" ✏️ 입력 모드 ")
                .border_style(Style::default().fg(Color::Cyan)),
        );

    frame.render_widget(input, area);
}

fn render_confirm_modal(frame: &mut Frame) {
    let area = centered_rect(40, 6, frame.area());
    frame.render_widget(Clear, area);

    let text = "정말 삭제하시겠습니까?\\n\\n  [y] 예  [n] 아니오";
    let confirm = Paragraph::new(text)
        .style(Style::default().fg(Color::Red))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" ⚠️ 확인 ")
                .border_style(Style::default().fg(Color::Red)),
        );

    frame.render_widget(confirm, area);
}

fn render_dir_picker(frame: &mut Frame, area: Rect, app: &mut App) {
    let is_searching = app.input_target == Some(super::app::InputTarget::DirPickerSearch);
    let matches = app.get_matching_indices();
    let selected_index = app.dir_picker.as_ref().map(|dp| dp.selected_index).unwrap_or(0);

    let dp = match &mut app.dir_picker {
        Some(dp) => dp,
        None => return,
    };
    
    let block = Block::default()
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(Color::DarkGray));
    
    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Current path
            Constraint::Length(2), // Subtitle
            Constraint::Min(3),    // List
            Constraint::Length(1), // spacing
            Constraint::Length(1), // Confirm
            Constraint::Length(1), // Cancel
            Constraint::Length(2), // Search
        ])
        .split(inner_area);

    // 1. Current path
    let current_path = Paragraph::new(format!(" 현재 위치: {}", dp.current_dir.display()))
        .style(Style::default().fg(Color::Cyan).bold());
    frame.render_widget(current_path, layout[0]);
    
    // Subtitle
    frame.render_widget(Paragraph::new(format!(" {}", app.status_message)).style(Style::default().fg(Color::DarkGray)), layout[1]);

    // 2. Directory list
    let items: Vec<ListItem> = dp.items.iter().enumerate().map(|(i, path)| {
        let is_selected = dp.selected_index == i;
        let is_checked = dp.selected_paths.contains(path);
        
        let prefix = if is_selected { "> " } else { "  " };
        let check = if is_checked { "[x]" } else { "[ ]" };
        
        let name = if path.file_name().map(|n| n == "..").unwrap_or(false) || path == &dp.current_dir.parent().unwrap_or(std::path::Path::new("")) {
            ".. (상위 폴더로)".to_string()
        } else {
            path.file_name().unwrap_or_default().to_string_lossy().to_string()
        };

        let style = if is_selected && dp.focus == super::app::DirPickerFocus::List {
            Style::default().fg(Color::Green).bold()
        } else if is_checked {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::White)
        };

        ListItem::new(format!(" {}{} {}", prefix, check, name)).style(style)
    }).collect();

    let list = List::new(items);
    frame.render_stateful_widget(list, layout[2], &mut dp.list_state);

    // 3. Confirm / Cancel buttons
    let confirm_style = if dp.focus == super::app::DirPickerFocus::Confirm {
        Style::default().fg(Color::Green).bold()
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let confirm_prefix = if dp.focus == super::app::DirPickerFocus::Confirm { "> " } else { "  " };
    let confirm_btn = Paragraph::new(format!(" {}선택 완료 ({}개)", confirm_prefix, dp.selected_paths.len()))
        .style(confirm_style);
    frame.render_widget(confirm_btn, layout[4]);

    let cancel_style = if dp.focus == super::app::DirPickerFocus::Cancel {
        Style::default().fg(Color::Green).bold()
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let cancel_prefix = if dp.focus == super::app::DirPickerFocus::Cancel { "> " } else { "  " };
    let cancel_btn = Paragraph::new(format!(" {}취소", cancel_prefix))
        .style(cancel_style);
    frame.render_widget(cancel_btn, layout[5]);

    // 4. Search area
    let search_text = if is_searching {
        if matches.is_empty() {
            format!(" 🔍 검색: {}▊ (매칭 없음)", app.input_buffer)
        } else {
            let current_pos = matches.iter().position(|&x| x == selected_index).map(|p| p + 1).unwrap_or(1);
            format!(" 🔍 검색: {}▊ ({}/{} 매칭)", app.input_buffer, current_pos, matches.len())
        }
    } else {
        " 🔍 '/' 키를 입력하여 실시간 검색".to_string()
    };

    let search_block = Paragraph::new(search_text)
        .style(if is_searching { Style::default().fg(Color::Green) } else { Style::default().fg(Color::DarkGray) });
    frame.render_widget(search_block, layout[6]);
}
"""

with open("src/tui/ui.rs", "w") as f:
    f.write(new_ui)

print("Generated src/tui/ui.rs")
