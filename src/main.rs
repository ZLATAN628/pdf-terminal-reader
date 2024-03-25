use pdf_terminal_reader::app::{App};
use pdf_terminal_reader::event::{Event, EventHandler};
use pdf_terminal_reader::handler::handle_key_events;
use pdf_terminal_reader::tui::Tui;
use std::io;
use clap::Parser;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use pdf_terminal_reader::{emit};
use pdf_terminal_reader::history::History;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct AppArgs {
    /// pdf path
    /// if None => last pdf
    #[arg(short, long)]
    path: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = AppArgs::parse();
    let default_path = String::from("/Users/zlatan/Documents/电子书/rust-book-zh-cn-shieber.pdf");
    let pdf_path = args.path.as_ref().unwrap_or(&default_path);
    let mut history = History::init();
    let mut app = App::new(pdf_path, history.read_last_page_num(pdf_path).unwrap_or(0));

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
                let next_page_id = if app.next_load_page <= app.pdf_handler.get_page_nums() as u32 {
                    Some(app.next_load_page)
                } else {
                    None
                };
                app.page_cache.load_next_page(app.pdf_handler.get_pdf_path(), next_page_id).await;
                app.next_load_page += 1;
            }
            Event::ChangeState(state) => {
                app.app_state = state;
            }
        }
    }
    // Exit the user interface.
    tui.exit()?;
    history.save_page_num(pdf_path, app.cur_page);
    Ok(())
}
