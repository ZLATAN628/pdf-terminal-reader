use crate::app::{App};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> anyhow::Result<()> {
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
        KeyCode::Char('w') | KeyCode::Char('W') => {
            app.book_marks_previous();
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            app.book_marks_next();
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
        // Counter handlers
        KeyCode::Right => {}
        KeyCode::Left => {}
        // Other handlers you could add here.
        _ => {}
    }
    Ok(())
}
