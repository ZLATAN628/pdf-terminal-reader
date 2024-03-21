use pdf_terminal_reader::app::{App};
use pdf_terminal_reader::event::{Event, EventHandler};
use pdf_terminal_reader::handler::handle_key_events;
use pdf_terminal_reader::tui::Tui;
use std::io;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use pdf_terminal_reader::emit;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ImageHandler::render_image(Rect { x: 40, y: 0, height: 1, width: 1 }, &Path::new("/Users/zlatan/Documents/电子书/test.jpeg")).await.unwrap();
    // tokio::time::sleep(Duration::from_millis(80000)).await;
    // return Ok(());
    // TODO parse args
    let pdf_path = "/Users/zlatan/Documents/电子书/rust-book-zh-cn-shieber.pdf";
    // Create an application.
    let mut app = App::new(pdf_path);

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(1000);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Start the main loop.
    while app.running {
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        let event = tui.events.next().await?;
        match event {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            // Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            Event::RenderPdf => {
                if !app.page_cache.page_exists(app.cur_page).await {
                    emit!(LoadingFirst(app.cur_page));
                    continue;
                }
                match app.page_cache.load_page_data(app.cur_page) {
                    Ok(data) => {
                        app.image_handler.render_image(&data,
                                                       &app.pdf_size)?;
                        app.loading = false;
                        // 继续静默加载
                        if app.next_load_page <= app.pdf_handler.get_page_nums() as u32 {
                            app.next_load_page += 1;
                            emit!(LoadingNext);
                        }
                    }
                    Err(e) => {
                        if let io::ErrorKind::NotFound = e.kind() {
                            emit!(LoadingFirst(app.cur_page));
                        }
                    }
                }
            }
            Event::LoadingFirst(page_id) => {
                app.page_cache.add_first(page_id).await;
            }
            Event::LoadingNext => {
                app.page_cache.load_next_page(app.pdf_handler.get_pdf_path(), app.next_load_page).await;
            }
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
