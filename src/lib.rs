extern crate core;

/// Application.
pub mod app;

/// Terminal events handler.
pub mod event;

/// Widget renderer.
pub mod ui;

/// Terminal user interface.
pub mod tui;

/// Event handler.
pub mod handler;

/// PDF handler
pub mod pdf;

/// character decode
pub mod decode;

/// image handler
pub mod image;

/// pdf page to jpeg cache
pub mod cache;
pub mod ro_cell;

/// PDF file read history
pub mod history;


#[cfg(test)]
mod tests {
    use crate::pdf::PdfHandler;
    use super::*;

    #[test]
    fn book_mark_test() {
        let pdf_handler = PdfHandler::new("/Users/zlatan/Documents/电子书/rust-book-zh-cn-shieber.pdf");
    }

    #[test]
    fn vec_test() {
        let a = vec![3; 5];
        let b = &a[0..];
        println!("{b:?}");
    }
}