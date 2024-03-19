use ratatui::{
    style::{Color, Style},
    widgets::{Block},
    Frame,
};
use ratatui::prelude::*;
use ratatui::widgets::{Borders, List, ListItem};
use tokio::sync::mpsc;

use crate::app::{App};
use crate::event::Event;
use crate::pdf::{BookMark, BookMarkIndex, PdfHandler};

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame, sender: mpsc::UnboundedSender<Event>) {
    let chunk = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Fill(1)])
        .split(frame.size());
    // left side => catalog
    let book_marks = app.pdf_handler.get_book_marks();
    let mut items: Vec<ListItem> = vec![];
    let mut index_vec: Vec<BookMarkIndex> = vec![];
    let mut cur_index: Vec<usize> = vec![];
    parse_book_marks_item(book_marks, &mut items, &mut index_vec, &mut cur_index);
    app.ui_book_marks = Some(index_vec);
    let list_widget = List::new(items)
        .block(Block::default().title(app.pdf_handler.get_title().as_str()).borders(Borders::RIGHT))
        .highlight_style(Style::new().red().italic())
        .highlight_symbol("*");
    frame.render_stateful_widget(list_widget, chunk[0], &mut app.book_marks_state);
    // right side => pdf preview
    if !app.already_render {
        app.already_render = true;
        let page_path = app.page_cache.get_page(app.cur_page);
        let pdf_path = app.pdf_handler.get_pdf_path().to_string();
        tokio::spawn(
            PdfHandler::render_pdf_page(
                pdf_path, page_path, app.cur_page, chunk[1].clone(), sender,
            )
        );
    }
}

fn parse_book_marks_item(book_marks: &Vec<BookMark>, items: &mut Vec<ListItem>,
                         index_vec: &mut Vec<Vec<usize>>, cur_index: &mut Vec<usize>) {
    let mut index = 0;
    for bm in book_marks {
        if !bm.is_show() {
            continue;
        }
        cur_index.push(index);
        let sub_symbol =
            if bm.is_sub_show() { String::from(" ▼") } else if !bm.get_sub().is_empty() { String::from(" ▶") } else { String::new() };
        let item = Line::from(Span::styled(
            format!("{}{}{}", " ".repeat(bm.get_hierarchy() as usize), bm.get_name(), sub_symbol),
            Style::default().fg(Color::Yellow),
        ));
        items.push(ListItem::new(item));
        index_vec.push(cur_index.clone());
        if !bm.get_sub().is_empty() {
            parse_book_marks_item(bm.get_sub(), items, index_vec, cur_index);
        }
        index += 1;
        cur_index.pop();
    }
}