use ratatui::{
    style::{Color, Style},
    widgets::{Block},
    Frame,
};
use ratatui::prelude::*;
use ratatui::widgets::{Borders, List, ListItem, Paragraph};

use crate::app::{App, AppState};
use crate::emit;
use crate::pdf::{BookMark, BookMarkIndex};

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    // left side => catalog
    let chunk = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Fill(1)])
        .split(frame.size());
    match &app.app_state {
        AppState::Search(text) => {
            let chunk = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Fill(1)])
                .split(chunk[0]);
            render_search_box(frame, chunk[0], text);
            render_catalog(app, frame, chunk[1]);
        }
        _ => {
            render_catalog(app, frame, chunk[0]);
        }
    }

    // right side => pdf preview
    let chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Fill(1)])
        .split(chunk[1]);

    render_pdf(app, chunk[1]);

    let page_id =
        if let AppState::JumpPage(page_id) = &app.app_state { Some(page_id.parse::<u32>().unwrap_or(0)) } else { None };
    render_title(app, frame, chunk[0], page_id);
}

fn render_search_box(frame: &mut Frame, chunk: Rect, text: &String) {
    let paragraph = Paragraph::new(Line::from(Span::styled(
        format!("{text}"),
        Style::default().green(),
    ))).block(Block::default().borders(Borders::ALL).border_style(Style::new().blue()));
    frame.render_widget(paragraph, chunk);
}

fn render_title(app: &mut App, frame: &mut Frame, chunk: Rect, page_id: Option<u32>) {
    let loading = if app.loading { String::from("加载中...") } else { String::new() };
    let mut line = Vec::new();
    if let Some(page_id) = page_id {
        line.push(Span::styled(
            "第 ",
            Style::default().green(),
        ));
        line.push(Span::styled(
            format!("{page_id}"),
            Style::default().red(),
        ));
        line.push(Span::styled(
            format!("/{} 页  {loading}", app.pdf_handler.get_page_nums()),
            Style::default().green(),
        ))
    } else {
        line.push(Span::styled(
            format!("第 {}/{} 页  {loading}", app.cur_page, app.pdf_handler.get_page_nums()),
            Style::default().green(),
        ));
    }
    let title = Paragraph::new(Line::from(line));

    frame.render_widget(title, chunk);
}

fn render_pdf(app: &mut App, chunk: Rect) {
    if !app.already_render {
        app.already_render = true;
        app.loading = true;
        app.pdf_size.update(&chunk);
        emit!(LoadingFirst(app.cur_page));
    }
}

fn render_catalog(app: &mut App, frame: &mut Frame, chunk: Rect) {
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
    frame.render_stateful_widget(list_widget, chunk, &mut app.book_marks_state);
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