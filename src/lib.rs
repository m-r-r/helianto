extern crate rustc_serialize;
extern crate walkdir;

mod utils;
mod error;

use std::path::PathBuf;
use std::default::Default;
use walkdir::WalkDir;

use utils::PathExt;
pub use error::{Error, Result};



#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct Settings {
    pub source_dir: PathBuf,
    pub dest_dir: PathBuf,
    pub base_url: String,
    pub max_depth: usize,
    pub follow_links: bool,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            source_dir: PathBuf::from("."),
            dest_dir: PathBuf::from("_output"),
            base_url: String::from("/"),
            max_depth: ::std::usize::MAX,
            follow_links: false,
        }
    }
}

pub struct Generator {
    pub settings: Settings,
}

impl Generator {
    pub fn new(settings: &Settings) -> Generator {
        Generator { settings: settings.clone() }
    }


    fn check_settings(&self) {
        let Settings { ref source_dir, ref dest_dir, .. } = self.settings;

        if !source_dir.is_dir() {
            panic!("{} is not a directory", source_dir.display());
        }

        if dest_dir.exists() && !dest_dir.is_dir() {
            panic!("{} must be a directory", dest_dir.display());
        }
    }

    pub fn run(&mut self) {
        self.check_settings();

        let entries = WalkDir::new(&self.settings.source_dir)
                          .max_depth(self.settings.max_depth)
                          .follow_links(self.settings.follow_links)
                          .into_iter();

        for entry in entries.filter(|entry| utils::valid_filename(entry.path.filename())) {
            if let Err(e) = entry {
                println!("{}", e);
                continue;
            }
            let entry = entry.unwrap();
            println!("{:?}", entry);
        }
    }
}


pub enum FileType {
    Document,
    Media,
    Asset,
}

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
