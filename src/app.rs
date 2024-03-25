use ratatui::widgets::ListState;
use crate::cache::FileCache;
use crate::image::ImageHandler;
use crate::pdf::{BookMarkIndex, PdfHandler, PdfSize};

#[derive(Debug, Clone)]
pub enum AppState {
    Normal,
    Search(String),
    JumpPage(String),
}

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
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
    /// loading pdf?
    pub loading: bool,
    /// pdf to jpeg page cache
    pub page_cache: FileCache,
    /// pdf preview width * height
    pub pdf_size: PdfSize,
    /// backend load page
    pub next_load_page: u32,
    /// state
    pub app_state: AppState,
}


impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(path: &str, last_page: u32) -> Self {
        Self {
            running: true,
            pdf_handler: PdfHandler::new(path),
            image_handler: ImageHandler::new(),
            book_marks_state: ListState::default(),
            ui_book_marks: None,
            cur_page: last_page,
            already_render: false,
            loading: true,
            page_cache: FileCache::new(path.to_string()),
            pdf_size: PdfSize::new(1200.0, 1500.0, 0, 0),
            next_load_page: 2,
            app_state: AppState::Normal,
        }
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&mut self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub(crate) fn book_marks_previous(&mut self, is_shift: bool) {
        if let Some(index) = self.book_marks_state.selected() {
            if is_shift {
                let ui_book_marks = self.ui_book_marks.as_ref().unwrap();
                let mut final_index = index;
                let origin_hierarchy = ui_book_marks[index].len();
                loop {
                    if final_index > 0 {
                        final_index -= 1;
                        let temp = &ui_book_marks[final_index];
                        if temp.len() < origin_hierarchy || (temp.len() == 1 && origin_hierarchy == 1) {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                self.book_marks_state.select(Some(final_index));
            } else {
                if index > 0 {
                    self.book_marks_state.select(Some(index - 1));
                }
            }
        } else {
            self.book_marks_state.select(Some(0));
        }
    }

    pub(crate) fn book_marks_next(&mut self, is_shift: bool) {
        if let Some(index) = self.book_marks_state.selected() {
            let ui_book_marks = self.ui_book_marks.as_ref().unwrap();
            if is_shift {
                let mut final_index = index;
                let origin_hierarchy = ui_book_marks[index].len();
                loop {
                    if final_index < ui_book_marks.len() - 1 {
                        final_index += 1;
                        let temp = &ui_book_marks[final_index];
                        if temp.len() < origin_hierarchy || (temp.len() == 1 && origin_hierarchy == 1) {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                self.book_marks_state.select(Some(final_index));
            } else {
                if index < ui_book_marks.len() - 1 {
                    self.book_marks_state.select(Some(index + 1));
                }
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
        if self.cur_page < self.pdf_handler.get_page_nums() as u32 {
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

    pub(crate) fn increment_pdf_size(&mut self) {
        self.already_render = false;
        self.pdf_size.increment();
    }

    pub(crate) fn decrement_pdf_size(&mut self) {
        self.already_render = false;
        self.pdf_size.decrement();
        // clear screen
    }

    #[allow(dead_code)]
    fn get_current_book_mark_index(&self) -> Option<&BookMarkIndex> {
        if let Some(index) = self.book_marks_state.selected() {
            if let Some(ui_book_marks) = self.ui_book_marks.as_ref() {
                return Some(&ui_book_marks[index]);
            }
        }
        None
    }
}
