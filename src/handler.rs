use crate::app::{App, AppState};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::emit;

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> anyhow::Result<()> {
    match &mut app.app_state {
        AppState::Normal => {
            deal_normal_key_event(key_event, app);
        }
        AppState::Search(text) => {
            deal_search_key_event(key_event, text);
        }
        AppState::JumpPage(_) => {
            deal_jump_page_key_event(app, key_event);
        }
    }
    Ok(())
}

fn deal_jump_page_key_event(app: &mut App, key_event: KeyEvent) {
    if let AppState::JumpPage(page_id) = &mut app.app_state {
        match key_event.code {
            KeyCode::Char(num) => {
                if num.is_numeric() {
                    page_id.push(num);
                }
            }
            KeyCode::Esc => {
                emit!(ChangeState(AppState::Normal))
            }
            KeyCode::Enter => {
                if let Ok(id) = page_id.parse::<u32>() {
                    if id > 0 && id <= app.pdf_handler.get_page_nums() as u32 {
                        app.cur_page = id;
                        emit!(RenderPdf);
                    }
                }
                emit!(ChangeState(AppState::Normal))
            }
            KeyCode::Backspace => {
                page_id.pop();
            }
            _ => {}
        }
    }
}

fn deal_normal_key_event(key_event: KeyEvent, app: &mut App) {
    match key_event.code {
        // Exit application on `ESC` or `q`
        KeyCode::Esc | KeyCode::Char('q') => {
            app.quit();
        }
        // Exit application on `Ctrl-C`
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit();
            }
        }
        KeyCode::Char('w') => {
            app.book_marks_previous(false);
        }
        KeyCode::Char('W') => {
            app.book_marks_previous(true);
        }
        KeyCode::Char('s') => {
            app.book_marks_next(false);
        }
        KeyCode::Char('S') => {
            app.book_marks_next(true);
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            app.toggle_bookmark_expansion(true);
        }
        KeyCode::Char('a') | KeyCode::Char('A') => {
            app.toggle_bookmark_expansion(false);
        }
        KeyCode::Enter => {
            app.jump_to_book_mark_page();
        }
        KeyCode::Down => {
            app.next_page();
        }
        KeyCode::Up => {
            app.previous_page();
        }
        KeyCode::Char('+') => {
            app.increment_pdf_size()
        }
        KeyCode::Char('-') => {
            app.decrement_pdf_size()
        }
        KeyCode::Char('/') => {
            emit!(ChangeState(AppState::Search(String::new())));
        }
        KeyCode::Char('.') => {
            emit!(ChangeState(AppState::JumpPage(format!("{}", app.cur_page))));
        }
        // Counter handlers
        KeyCode::Right => {}
        KeyCode::Left => {}
        // Other handlers you could add here.
        _ => {}
    }
}

fn deal_search_key_event(key_event: KeyEvent, text: &mut String) {
    match key_event.code {
        KeyCode::Enter => {
            // TODO search
            emit!(ChangeState(AppState::Normal))
        }
        KeyCode::Char(c) => {
            text.push(c);
        }
        KeyCode::Backspace => {
            text.pop();
        }
        KeyCode::Esc => {
            emit!(ChangeState(AppState::Normal))
        }
        _ => {}
    }
}

