use std::collections::BTreeMap;
use std::ffi::OsString;
use std::ops::Index;
use std::path::Path;
use lopdf::{Document, Object, ObjectId, Outline};
use ratatui::prelude::Rect;
use crate::decode::decode_str_to_utf8;


#[derive(Debug, Clone)]
pub struct BookMarkIndex {
    inner: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct PdfSize {
    x: u16,
    y: u16,
    width: f32,
    height: f32,
}

#[derive(Debug)]
pub struct PdfHandler {
    document: Document,
    // 页映射 对象Id => 页数
    page_map: BTreeMap<ObjectId, u32>,
    // 解析后的书签集合
    book_marks: Vec<BookMark>,
    // pdf 文件路径
    pdf_path: String,
    // 总页数
    page_nums: usize,
    // pdf 书籍标题
    title: String,
}

#[derive(Debug, Clone, Default)]
pub struct BookMark {
    /// 书签名
    name: String,
    /// 第几页
    num: u32,
    /// 子目录
    sub: Vec<BookMark>,
    /// 目录层级
    hierarchy: u32,
    /// 是否展示
    pub show: bool,
    /// 子目录是否展示
    pub sub_show: bool,
}

impl BookMark {
    pub fn name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn num(mut self, num: u32) -> Self {
        self.num = num;
        self
    }

    pub fn get_num(&self) -> u32 {
        self.num
    }

    pub fn get_sub(&self) -> &Vec<BookMark> {
        &self.sub
    }

    pub fn get_sub_mut(&mut self) -> &mut Vec<BookMark> {
        &mut self.sub
    }

    pub fn hierarchy(mut self, hierarchy: u32) -> Self {
        self.hierarchy = hierarchy;
        self
    }

    pub fn get_hierarchy(&self) -> u32 {
        self.hierarchy
    }

    pub fn show(mut self, show: bool) -> Self {
        self.show = show;
        self
    }

    pub fn is_show(&self) -> bool {
        self.show
    }

    pub fn is_sub_show(&self) -> bool {
        self.sub_show
    }
}

impl PdfHandler {
    pub fn new(path: &str) -> Self {
        let document = Document::load(path).expect("pdf file not found");
        let title = OsString::from(Path::new(path).file_stem().unwrap()).into_string().unwrap();
        // TODO parse title
        // if let Ok(Object::Reference(info_id)) = document.trailer.get(b"Info") {
        //     if let Ok(info) = document.get_dictionary(info_id.clone()) {
        //         println!("{info:?}");
        //     }
        // }
        let page_map: BTreeMap<ObjectId, u32> = document.page_iter().enumerate().map(|(i, p)| (p, i as u32 + 1)).collect();
        let page_nums = page_map.len();
        let mut pdf_handler = Self {
            document,
            page_map,
            book_marks: vec![],
            pdf_path: path.to_string(),
            page_nums,
            title,
        };
        pdf_handler.init();
        pdf_handler
    }

    fn init(&mut self) {
        self.parse_book_marks();
    }

    pub fn get_book_marks(&self) -> &Vec<BookMark> {
        &self.book_marks
    }

    pub fn get_pdf_path(&self) -> &str {
        &self.pdf_path
    }

    pub fn get_page_nums(&self) -> usize {
        self.page_nums
    }

    pub fn get_title(&self) -> &String {
        &self.title
    }

    pub fn find_book_mark(&self, index: &BookMarkIndex) -> Option<&BookMark> {
        let mut bms: &Vec<BookMark> = &self.book_marks;
        for i in 0..index.len() {
            if let Some(bm) = bms.get(index[i]) {
                if i == index.len() - 1 {
                    return Some(bm);
                }
                bms = &bm.sub;
            } else {
                break;
            }
        }
        None
    }

    pub fn find_book_mark_mut(&mut self, index: &BookMarkIndex) -> Option<&mut BookMark> {
        let mut bms: &mut Vec<BookMark> = &mut self.book_marks;
        for i in 0..index.len() {
            if let Some(bm) = bms.get_mut(index[i]) {
                if i == index.len() - 1 {
                    return Some(bm);
                }
                bms = &mut bm.sub;
            } else {
                break;
            }
        }
        None
    }

    /// 获取书签目录
    fn parse_book_marks(&mut self) {
        let mut map = BTreeMap::new();
        let mut book_marks = vec![];
        if let Some(outlines) = self.document.get_outlines(None, None, &mut map).unwrap() {
            self.parse_outlines(&outlines, &mut book_marks, 0);
        }
        book_marks.push(BookMark::default().show(true));
        self.book_marks = book_marks;
    }

    pub fn parse_outlines(&self, outlines: &Vec<Outline>, book_marks: &mut Vec<BookMark>, hierarchy: u32) {
        for outline in outlines.iter() {
            let mut book_mark = BookMark::default();
            match outline {
                Outline::Destination(dest) => {
                    let title = dest.title().unwrap();
                    let title = if let Object::String(bytes, _) = title {
                        decode_str_to_utf8(bytes).unwrap_or("unknown".to_owned())
                    } else {
                        "unknown".to_owned()
                    };
                    let page = dest.page().unwrap();
                    let mut num = 0;
                    if let Object::Reference(id) = page {
                        num = *self.page_map.get(id).unwrap();
                    }
                    book_mark = book_mark
                        .name(title)
                        .hierarchy(hierarchy)
                        .num(num)
                        .show(hierarchy == 0);
                }
                Outline::SubOutlines(os) => {
                    let mut sub_marks = vec![];
                    self.parse_outlines(os, &mut sub_marks, hierarchy + 1);
                    if let Some(last) = book_marks.last_mut() {
                        last.sub = sub_marks;
                    }
                    continue;
                }
            }
            book_marks.push(book_mark);
        }
    }
}

impl PdfSize {
    pub fn new(width: f32, height: f32, x: u16, y: u16) -> Self {
        Self {
            width,
            height,
            x,
            y,
        }
    }
    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn x(&self) -> u16 {
        self.x
    }

    pub fn y(&self) -> u16 {
        self.y
    }

    pub(crate) fn increment(&mut self) {
        self.width *= 1.1;
        self.height *= 1.1;
    }

    pub(crate) fn decrement(&mut self) {
        self.width *= 0.9;
        self.height *= 0.9;
    }

    pub(crate) fn update(&mut self, rect: &Rect) {
        self.x = rect.x;
        self.y = rect.y;
    }
}

impl From<Vec<usize>> for BookMarkIndex {
    fn from(value: Vec<usize>) -> Self {
        Self {
            inner: value
        }
    }
}

impl From<&[usize]> for BookMarkIndex {
    fn from(value: &[usize]) -> Self {
        Self {
            inner: Vec::from(value)
        }
    }
}

impl Index<usize> for BookMarkIndex {
    type Output = usize;
    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

impl BookMarkIndex {
    pub fn parent(&self) -> BookMarkIndex {
        if self.inner.len() > 1 {
            BookMarkIndex::from(&self.inner[0..self.inner.len() - 1])
        } else {
            self.clone()
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    // pub fn next(mut self, pdf_handler: &PdfHandler) -> Option<&BookMark> {
    //     let mut inner: &mut [usize] = &mut self.inner;
    //     loop {
    //         let last = inner.last_mut().unwrap();
    //         *last += 1;
    //         match pdf_handler.find_book_mark(inner) {
    //             Some(book_mark) => return Some(book_mark),
    //             None => {
    //                 if inner.len() == 1 {
    //                     return None;
    //                 }
    //                 inner = &mut inner[0..inner.len() - 1];
    //             }
    //         }
    //     }
    //
    //     // if inner.len() > 1 {}
    // }
}