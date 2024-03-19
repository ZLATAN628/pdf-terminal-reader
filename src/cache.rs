use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use crate::event::Event;


#[derive(Debug)]
pub struct FileCache {
    path: PathBuf,
    page_cache: HashMap<u32, bool>,
}

impl FileCache {
    pub fn new(path: String) -> Self {
        let path = PathBuf::from(&path);
        let parent = path.parent().unwrap();
        let file_name = path.file_stem().unwrap();
        let path = parent.join(format!("{}-rpr", OsString::from(file_name).into_string().unwrap()));
        let page_cache = Self::init_page_cache(&path);
        Self {
            path,
            page_cache,
        }
    }

    pub fn init_page_cache(path: &PathBuf) -> HashMap<u32, bool> {
        if !path.exists() {
            fs::create_dir(&path).expect("error to create pdf directory");
        }
        let mut cache = HashMap::new();
        for entry in path.read_dir().expect("read_dir call failed") {
            if let Ok(entry) = entry {
                let entry_path = entry.path();
                let page_id = OsString::from(entry_path.file_stem().unwrap()).into_string().unwrap();
                if let Ok(page_id) = page_id.parse::<u32>() {
                    cache.insert(page_id, true);
                }
            }
        }
        cache
    }

    pub fn save_page(&mut self, page_id: u32) {
        self.page_cache.insert(page_id, true);
    }

    pub fn get_page(&self, page_id: u32) -> (String, bool) {
        let path = self.get_page_path(page_id);
        if let Some(exist) = self.page_cache.get(&page_id) {
            if *exist {
                return (path, true);
            }
        }
        (path, false)
    }

    #[inline]
    pub fn get_page_path(&self, page_id: u32) -> String {
        format!("{}/{page_id}.jpeg", self.path.display())
    }

    pub fn convert_pdf_to_ppm(pdf_path: &str, page_path: &str, page_id: u32, sender: mpsc::UnboundedSender<Event>) -> Vec<u8> {
        let data = Command::new("pdftoppm")
            .args(["-jpeg", "-jpegopt", "quality=75", "-f", &format!("{page_id}"), pdf_path])
            .output()
            .expect("convert pdf failed");
        let image_data = data.stdout;
        fs::write(Path::new(page_path), &image_data).expect("write page data error");
        sender.send(Event::SavePdfPage(page_id)).expect("send save pdf page message error");
        image_data
    }
}