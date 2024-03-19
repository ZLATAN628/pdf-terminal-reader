use std::fs;
use std::io::{stderr, stdout, Write};
use std::path::Path;
use base64::Engine;
use base64::engine::general_purpose;
use crossterm::cursor::{MoveTo, RestorePosition, SavePosition};
use crossterm::queue;
use ratatui::prelude::Rect;

#[derive(Debug, Clone)]
pub struct ImageHandler {
    image_area: Option<Rect>,
}

impl ImageHandler {
    pub fn new() -> Self {
        Self {
            image_area: None
        }
    }

    pub fn set_image_area(&mut self, image_area: Rect) {
        self.image_area = Some(image_area);
    }


    pub async fn render_image(area: Rect, image_data: &[u8]) -> anyhow::Result<()> {
        let b64 = general_purpose::STANDARD.encode(image_data);
        let mut buf = vec![];
        write!(buf, "\x1b]1337;File=inline=1;size={};width={}px;height={}px;doNotMoveCursor=1:{}\x07",
               image_data.len(),
               2000,
               3000,
               b64
        )?;
        move_lock(stdout().lock(), (area.x, area.y), |stdout| {
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