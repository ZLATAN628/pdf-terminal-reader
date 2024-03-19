use pdf_terminal_reader::app::{App, AppResult};
use pdf_terminal_reader::event::{Event, EventHandler};
use pdf_terminal_reader::handler::handle_key_events;
use pdf_terminal_reader::tui::Tui;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;
use ratatui::backend::CrosstermBackend;
use ratatui::prelude::Rect;
use ratatui::Terminal;
use pdf_terminal_reader::image::ImageHandler;
use pdf_terminal_reader::pdf::PdfHandler;

#[tokio::main]
async fn main() -> AppResult<()> {
    // ImageHandler::render_image(Rect { x: 40, y: 0, height: 1, width: 1 }, &Path::new("/Users/zlatan/Documents/电子书/test.jpeg")).await.unwrap();
    // tokio::time::sleep(Duration::from_millis(80000)).await;
    // return Ok(());
    // TODO parse args
    let pdf_path = "/Users/zlatan/Documents/电子书/程序员的自我修养--链接、装载与库(高清带完整书签版).pdf";
    // Create an application.
    let mut app = App::new(pdf_path);

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(100000);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Start the main loop.
    while app.running {
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        match tui.events.next().await? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            Event::SavePdfPage(page_id) => {
                app.page_cache.save_page(page_id)
            }
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
