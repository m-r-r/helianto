use std::path::PathBuf;

pub enum FileType { Document, Media, Asset }

pub struct FileInfo {
    kind: FileType,
    path: PathBuf,
}

pub struct DocumentInfo {
    pub title: String,
    pub language: Option<String>,
    pub modified: Option<u64>,
    pub created: u64,
    pub keywords: Vec<String>,
}

pub struct Document {
    pub info: DocumentInfo,
    pub source: FileInfo,
    pub content: String,
}


pub struct Media {
    pub info: DocumentInfo,
    pub source: FileInfo,
    pub content: String,
}

