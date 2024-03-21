use std::io::{stdout, Write};
use base64::Engine;
use base64::engine::general_purpose;
use crossterm::cursor::{MoveTo, RestorePosition, SavePosition};
use crossterm::queue;
use ratatui::prelude::Rect;
use tokio::sync::{Mutex};
use crate::pdf::PdfSize;

#[derive(Debug)]
pub struct ImageHandler {
    image_area: Option<Rect>,
    pdf_lock: Mutex<()>,
}

impl ImageHandler {
    pub fn new() -> Self {
        Self {
            image_area: None,
            pdf_lock: Mutex::new(()),
        }
    }

    pub fn set_image_area(&mut self, image_area: Rect) {
        self.image_area = Some(image_area);
    }


    pub fn render_image(&self, image_data: &[u8], pdf_size: &PdfSize) -> anyhow::Result<()> {
        let b64 = general_purpose::STANDARD.encode(image_data);
        let mut buf = vec![];
        write!(buf, "\x1b]1337;File=inline=1;size={};width={}px;height={}px;doNotMoveCursor=1:{}\x07",
               image_data.len(),
               pdf_size.width(),
               pdf_size.height(),
               b64
        )?;
        let _lock = self.pdf_lock.lock();
        move_lock(stdout().lock(), (pdf_size.x(), pdf_size.y()), |stdout| {
            stdout.write_all(&buf)?;
            Ok(1)
        })?;
        Ok(())
    }
}


#[inline]
pub fn move_lock<W, F, T>(mut stdout: W, (x, y): (u16, u16), cb: F) -> anyhow::Result<T>
    where
        W: Write,
        F: FnOnce(&mut W) -> anyhow::Result<T>,
{
    #[cfg(unix)]
    {
        queue!(&mut stdout, SavePosition, MoveTo(x, y))?;
        let result = cb(&mut stdout);
        queue!(&mut stdout, RestorePosition)?;
        stdout.flush()?;
        result
    }
}