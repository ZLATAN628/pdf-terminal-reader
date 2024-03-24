use std::collections::HashMap;
use std::ffi::OsString;
use std::{fs, io};
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::Command;
use image::codecs::jpeg::JpegEncoder;
use image::io::Reader as ImageReader;
use crate::emit;


#[derive(Debug)]
pub struct FileCache {
    path: PathBuf,
    page_queue: Vec<u32>,
    cache: HashMap<u32, bool>,
}

impl FileCache {
    pub fn new(path: String) -> Self {
        let path = PathBuf::from(&path);
        let parent = path.parent().unwrap();
        let file_name = path.file_stem().unwrap();
        let path = parent.join(format!("{}-rpr", OsString::from(file_name).into_string().unwrap()));
        let cache = Self::init_page_cache(&path);
        Self {
            path,
            page_queue: Vec::new(),
            cache,
        }
    }

    pub fn init_page_cache(path: &PathBuf) -> HashMap<u32, bool> {
        let mut cache = HashMap::new();
        if !path.exists() {
            fs::create_dir(&path).expect("error to create pdf directory");
        }
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

    pub async fn page_exists(&self, page_id: u32) -> bool {
        return *self.cache.get(&page_id).unwrap_or(&false);
    }

    pub fn get_page_path(&self, page_id: u32) -> String {
        format!("{}/{page_id}.jpeg", self.path.display())
    }

    pub fn load_page_data(&self, page_id: u32) -> io::Result<Vec<u8>> {
        fs::read(&self.get_page_path(page_id))
    }

    pub async fn load_next_page(&mut self, pdf_path: &str, next_page_id: Option<u32>) {
        if let Some(id) = self.page_queue.first() {
            let id = *id;
            self.page_queue.remove(0);
            let page_path = self.get_page_path(id);
            self.convert_pdf_to_ppm(pdf_path.to_string(), page_path, id).await.unwrap();
        }
        if let Some(next_page_id) = next_page_id {
            self.page_queue.push(next_page_id);
        }
    }

    pub async fn add_first(&mut self, page_id: u32) {
        if self.page_exists(page_id).await {
            emit!(RenderPdf);
        } else {
            self.page_queue.insert(0, page_id);
            emit!(LoadingNext);
        }
    }

    pub async fn convert_pdf_to_ppm(&mut self, pdf_path: String, page_path: String, page_id: u32) -> anyhow::Result<()> {
        if let Some(page) = self.cache.get(&page_id) {
            if *page {
                emit!(RenderPdf);
                // already convert
                return Ok(());
            }
        }
        self.cache.insert(page_id, true);
        tokio::spawn(async move {
            let data = Command::new("pdftoppm")
                .args(["-jpeg", "-jpegopt", "quality=70", "-f", &format!("{page_id}"), &pdf_path.clone()])
                .output()
                .expect("convert pdf failed");
            let image_data = data.stdout;
            let img = ImageReader::new(Cursor::new(image_data)).with_guessed_format().unwrap().decode().unwrap();
            let mut jpg = vec![];
            match JpegEncoder::new_with_quality(&mut jpg, 70).encode_image(&img) {
                Ok(_) => {}
                Err(e) => {
                    // TODO error user tip
                    println!("convert image error => {e}");
                }
            };
            fs::write(Path::new(&page_path), &jpg).expect("write page data error");
            emit!(RenderPdf);
        });
        Ok(())
    }
}