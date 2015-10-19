extern crate rustc_serialize;
extern crate walkdir;
#[macro_use(wrap)]
extern crate hoedown;

mod utils;
mod error;
pub mod readers;

use std::path::{Path, PathBuf};
use std::default::Default;
use std::fs;
use walkdir::{WalkDir, WalkDirIterator};
use std::rc::Rc;
use std::collections::HashMap;

use utils::PathExt;
pub use error::{Error, Result};
use readers::Reader;


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
    readers: HashMap<String, Rc<Reader>>,
}

impl Generator {
    pub fn new(settings: &Settings) -> Generator {
        let mut generator = Generator {
            settings: settings.clone(),
            readers: HashMap::new(),
        };
        generator.add_reader::<readers::MarkdownReader>();
        generator
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

    pub fn get_reader(&self, path: &Path) -> Option<Rc<Reader>> {
        path.extension().and_then(|extension| extension.to_str())
            .and_then(|extension_str| self.readers.get(extension_str))
            .map(|rc| rc.clone())
    }

    pub fn add_reader<T: Reader + 'static>(&mut self) {
        let reader = Rc::new(T::new(&self.settings));

        for &extension in T::extensions() {
            self.readers.insert(extension.into(), reader.clone());
        }
    }


    fn render_document(&mut self, reader: Rc<Reader>, path: &Path) -> Result<()> {
        let (body, metadata) = try! { reader.load(path) }; 
        println!("{}", body);
        Ok(())
    }

    fn render_file(&mut self, path: &Path) -> Result<()> {
        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        self.check_settings();

        let entries = WalkDir::new(&self.settings.source_dir)
                          .max_depth(self.settings.max_depth)
                          .follow_links(self.settings.follow_links)
                          .into_iter();

        for entry in entries.filter_entry(|entry| utils::valid_filename(entry.path())) {
            if let Err(e) = entry {
                println!("{}", e);
                continue;
            }
            let entry = entry.map(|e| PathBuf::from(e.path())).unwrap();

            try! {
                match self.get_reader(&entry) {
                    Some(reader) => self.render_document(reader.clone(), &entry),
                    None => self.render_file(&entry),
                }
            }
        }

        Ok(())
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
