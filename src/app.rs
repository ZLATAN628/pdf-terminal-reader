use std::error;
use ratatui::widgets::ListState;
use crate::cache::FileCache;
use crate::image::ImageHandler;
use crate::pdf::{BookMarkIndex, PdfHandler};

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// counter
    pub counter: u8,
    /// pdf handler
    pub pdf_handler: PdfHandler,
    /// image handler
    pub image_handler: ImageHandler,
    /// List State
    pub book_marks_state: ListState,
    /// list
    pub ui_book_marks: Option<Vec<BookMarkIndex>>,
    /// current page
    pub cur_page: u32,
    /// Is the pdf already render?
    pub already_render: bool,
    /// pdf to jpeg page cache
    pub page_cache: FileCache,
}


impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(path: &str) -> Self {
        Self {
            running: true,
            counter: 0,
            pdf_handler: PdfHandler::new(path),
            image_handler: ImageHandler::new(),
            book_marks_state: ListState::default(),
            ui_book_marks: None,
            cur_page: 0,
            already_render: false,
            page_cache: FileCache::new(path.to_string()),
        }
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn increment_counter(&mut self) {
        if let Some(res) = self.counter.checked_add(1) {
            self.counter = res;
        }
    }

    pub fn decrement_counter(&mut self) {
        if let Some(res) = self.counter.checked_sub(1) {
            self.counter = res;
        }
    }

    pub(crate) fn book_marks_previous(&mut self) {
        if let Some(index) = self.book_marks_state.selected() {
            if index > 0 {
                self.book_marks_state.select(Some(index - 1));
            }
        } else {
            self.book_marks_state.select(Some(0));
        }
    }

    pub(crate) fn book_marks_next(&mut self) {
        if let Some(index) = self.book_marks_state.selected() {
            if index < self.ui_book_marks.as_ref().unwrap().len() - 1 {
                self.book_marks_state.select(Some(index + 1));
            }
        } else {
            self.book_marks_state.select(Some(0));
        }
    }

    pub(crate) fn toggle_bookmark_expansion(&mut self, show: bool) {
        if let Some(index) = self.book_marks_state.selected() {
            if let Some(ui_book_marks) = self.ui_book_marks.as_ref() {
                let index = &ui_book_marks[index];
                let book_mark = self.pdf_handler.find_book_mark_mut(index).unwrap();
                if !book_mark.get_sub().is_empty() {
                    book_mark.sub_show = show;
                    for bm in book_mark.get_sub_mut().iter_mut() {
                        bm.show = show;
                    }
                }
            }
        }
    }

    pub(crate) fn jump_to_book_mark_page(&mut self) {
        if let Some(index) = self.book_marks_state.selected() {
            if let Some(ui_book_marks) = self.ui_book_marks.as_ref() {
                let index = &ui_book_marks[index];
                let book_mark = self.pdf_handler.find_book_mark(index).unwrap();
                self.cur_page = book_mark.get_num();
                self.already_render = false;
            }
        }
    }

    pub(crate) fn next_page(&mut self) {
        if self.cur_page < self.pdf_handler.get_page_nums() as u32 - 1 {
            self.cur_page += 1;
            self.already_render = false;
        }
    }

    pub(crate) fn previous_page(&mut self) {
        if self.cur_page > 0 {
            self.cur_page -= 1;
            self.already_render = false;
        }
    }
}
