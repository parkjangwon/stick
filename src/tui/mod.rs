// ============================================================================
// stick - TUI 모듈 진입점
// ============================================================================

pub mod app;
pub mod ui;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;

use crate::config::StickConfig;
use app::App;

/// TUI 설정 화면 실행
pub fn run_config_tui() -> Result<()> {
    // 설정 로드
    let config = StickConfig::load()?;

    // 터미널 raw 모드 진입
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // TUI 앱 실행
    let mut app = App::new(config);
    let result = run_app(&mut terminal, &mut app);

    // 터미널 복원 (에러 발생해도 반드시 복원)
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // 실행 결과 처리
    if let Err(err) = result {
        eprintln!("TUI 에러: {:?}", err);
        return Err(err);
    }

    // 저장 여부 확인
    if app.should_save {
        app.config.save()?;
        println!("✅ 설정이 저장되었습니다.");
    } else {
        println!("ℹ️  변경사항이 저장되지 않았습니다.");
    }

    Ok(())
}

/// TUI 메인 이벤트 루프
fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        // 화면 렌더링
        terminal.draw(|frame| ui::render(frame, app))?;

        // 키 입력 처리
        if let Event::Key(key) = event::read()? {
            // key press 이벤트만 처리 (release 무시)
            if key.kind != KeyEventKind::Press {
                continue;
            }

            // 텍스트 입력 모드일 때
            if app.input_mode {
                match key.code {
                    KeyCode::Enter => app.submit_input(),
                    KeyCode::Esc => app.cancel_input(),
                    KeyCode::Char(c) => app.input_buffer.push(c),
                    KeyCode::Backspace => { app.input_buffer.pop(); }
                    _ => {}
                }
                continue;
            }

            // 삭제 확인 모드일 때
            if app.confirm_delete {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => app.confirm_delete_yes(),
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        app.confirm_delete = false;
                    }
                    _ => {}
                }
                continue;
            }

            // 일반 네비게이션 모드
            match key.code {
                KeyCode::Tab => {
                    if app.current_screen == app::Screen::DirPicker {
                        if let Some(dp) = &mut app.dir_picker {
                            dp.focus = match dp.focus {
                                app::DirPickerFocus::List => app::DirPickerFocus::Confirm,
                                app::DirPickerFocus::Confirm => app::DirPickerFocus::Cancel,
                                app::DirPickerFocus::Cancel => app::DirPickerFocus::List,
                            };
                        }
                    }
                }
                KeyCode::Char('q') => {
                    // 메인 메뉴에서만 종료 가능
                    if app.current_screen == app::Screen::Main {
                        break;
                    } else {
                        app.go_back();
                    }
                }
                KeyCode::Esc => app.go_back(),
                KeyCode::Up | KeyCode::Char('k') => app.move_up(),
                KeyCode::Down | KeyCode::Char('j') => app.move_down(),
                KeyCode::Enter => {
                    if app.current_screen == app::Screen::DirPicker {
                        app.select_dir_picker();
                    } else {
                        app.select();
                    }
                }
                KeyCode::Char('a') => app.handle_add(),
                KeyCode::Char('d') => app.handle_delete(),
                KeyCode::Char(' ') => {
                    if app.current_screen == app::Screen::DirPicker {
                        app.toggle_dir_picker_selection();
                    } else {
                        app.toggle_current();
                    }
                }
                KeyCode::Char('s') => {
                    // 어디서든 저장 가능
                    app.should_save = true;
                    app.config.save()?;
                    app.status_message = "✅ 설정이 저장되었습니다.".to_string();
                }
                _ => {}
            }
        }
    }
    Ok(())
}
