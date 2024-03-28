use std::collections::HashMap;
use std::{env, fs};
use std::path::{PathBuf};
use directories::BaseDirs;
use serde_json::{json, Value};

/// file name
const FILE_NAME: &'static str = "history";
/// Dir name
const DIR_NAME: &'static str = "pdf-terminal-reader";

/// {
///     "page": {
///         "pdf_path": $page_num
///     },
///     "last-read": $pdf_path
/// }
const PAGE: &'static str = "page";
const LAST_READ: &'static str = "last-read";

pub struct History {
    /// pdf name => last read page num
    page_record: Option<HashMap<String, Value>>,
    file_path: PathBuf,
}

impl History {
    pub fn init() -> Self {
        let base_dir = BaseDirs::new().unwrap();
        let data_dir = base_dir.data_dir();
        let pdf_history_path = data_dir.join(DIR_NAME);
        if !pdf_history_path.exists() {
            fs::create_dir(&pdf_history_path).unwrap();
        }
        let pdf_history_path = pdf_history_path.join(FILE_NAME);
        let page_record = if pdf_history_path.exists() {
            let history = fs::read_to_string(&pdf_history_path).expect("read history file failed");
            let page_record = serde_json::from_str::<HashMap<String, Value>>(&history).expect("parse history failed");
            Some(page_record)
        } else {
            None
        };
        Self {
            page_record,
            file_path: pdf_history_path,
        }
    }
    pub fn read_last_page_num(&self, pdf_path: &str) -> Option<u32> {
        if let Some(history) = self.page_record.as_ref() {
            let mut file_name = PathBuf::from(pdf_path);
            if file_name.is_relative() {
                file_name = env::current_dir().unwrap().join(file_name);
            }
            let file_name = file_name.display().to_string();
            if let Some(page_history) = history.get(PAGE) {
                if let Some(Value::Number(page_num)) = page_history.get(&file_name) {
                    if let Some(page_num) = page_num.as_u64() {
                        return Some(page_num as u32);
                    }
                }
            }
        }
        None
    }

    pub fn save_history(&mut self, pdf_path: &str, page_num: u32) {
        let mut file_name = PathBuf::from(pdf_path);
        if file_name.is_relative() {
            file_name = env::current_dir().unwrap().join(file_name);
        }
        let file_name = file_name.display().to_string();
        if let Some(history) = self.page_record.as_mut() {
            history.insert(String::from(LAST_READ), json!(&file_name));
            if let Some(Value::Object(page_history)) = history.get_mut(PAGE) {
                page_history.insert(file_name, json!(page_num));
                let data = serde_json::to_vec(history).unwrap();
                fs::write(&self.file_path, data).unwrap();
            }
        } else {
            let mut history: HashMap<String, Value> = HashMap::new();
            history.insert(String::from(LAST_READ), json!(&file_name));
            let mut page_history: HashMap<String, Value> = HashMap::new();
            page_history.insert(file_name, json!(page_num));
            history.insert(PAGE.into(), json!(page_history));
            let data = serde_json::to_vec(&history).unwrap();
            fs::write(&self.file_path, data).unwrap();
        }
    }

    pub fn get_last_read_pdf(&self) -> Option<String> {
        if let Some(history) = self.page_record.as_ref() {
            if let Some(Value::String(pdf)) = history.get(LAST_READ) {
                return Some(pdf.clone());
            }
        }
        None
    }
}