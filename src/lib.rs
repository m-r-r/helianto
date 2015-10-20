extern crate rustc_serialize;
extern crate walkdir;
#[macro_use(wrap)]
extern crate hoedown;

mod utils;
mod error;
pub mod readers;

use std::path::{Path, PathBuf};
use std::default::Default;
use std::error::Error as StdError;
use std::fs;
use std::fs::File;
use std::io::Write;
use walkdir::{WalkDir, WalkDirIterator};
use std::rc::Rc;
use std::collections::HashMap;

use utils::PathExt;
pub use error::{Error, Result};
use readers::Reader;


#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct Settings {
    pub source_dir: PathBuf,
    pub output_dir: PathBuf,
    pub base_url: String,
    pub max_depth: usize,
    pub follow_links: bool,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            source_dir: PathBuf::from("."),
            output_dir: PathBuf::from("_output"),
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
        let Settings { ref source_dir, ref output_dir, .. } = self.settings;

        if !source_dir.is_dir() {
            panic!("{} is not a directory", source_dir.display());
        }

        if output_dir.exists() && !output_dir.is_dir() {
            panic!("{} must be a directory", output_dir.display());
        }
    }

    pub fn get_reader(&self, path: &Path) -> Option<Rc<Reader>> {
        path.extension()
            .and_then(|extension| extension.to_str())
            .and_then(|extension_str| self.readers.get(extension_str))
            .map(|rc| rc.clone())
    }

    pub fn add_reader<T: Reader + 'static>(&mut self) {
        let reader = Rc::new(T::new(&self.settings));

        for &extension in T::extensions() {
            self.readers.insert(extension.into(), reader.clone());
        }
    }

    fn load_templates(&mut self) -> Result<()> {
        Ok(())
    }

    fn render_document(&mut self, reader: Rc<Reader>, path: &Path) -> Result<()> {
        let (body, metadata) = try! { reader.load(path) };
        let dest = path.relative_from_(&self.settings.source_dir)
                       .map(|relpath| relpath.with_extension("html"))
                       .unwrap();

        let mut info = DocumentInfo::default();



        let output: String = body.into();

        let dest_file = self.settings.output_dir.join(dest);
        let dest_dir = dest_file.parent().unwrap();
        fs::create_dir_all(&dest_dir)
            .and_then(|_| {
                let mut fd = try! { File::create(&dest_file) };
                try! { fd.write(output.as_ref()) };
                try! { fd.sync_data() };
                Ok(())
            })
            .map_err(|err| {
                Error::Output {
                    dest: dest_dir.into(),
                    cause: Box::new(err),
                }
            })
    }

    fn render_file(&mut self, path: &Path) -> Result<()> {
        let dest = path.relative_from_(&self.settings.source_dir)
                       .map(|relpath| self.settings.output_dir.join(relpath))
                       .unwrap();
        let dest_dir = dest.parent().unwrap();

        fs::create_dir_all(&dest_dir)
            .and_then(|_| fs::copy(path, &dest))
            .and_then(|_| {
                println!("{} -> {}", path.display(), dest.display());
                Ok(())
            })
            .map_err(|err| {
                Error::Copy {
                    from: path.into(),
                    to: dest_dir.into(),
                    cause: Box::new(err),
                }
            })
    }

    pub fn run(&mut self) -> Result<()> {
        self.check_settings();

        let entries = WalkDir::new(&self.settings.source_dir)
                          .max_depth(self.settings.max_depth)
                          .follow_links(self.settings.follow_links)
                          .into_iter();

        for entry in entries.filter_entry(|entry| utils::valid_filename(entry.path())) {
            let entry = match entry {
                Err(e) => continue,
                Ok(e) => {
                    if e.file_type().is_dir() {
                        continue
                    } else {
                        PathBuf::from(e.path())
                    }
                }
            };

            let result = match self.get_reader(&entry) {
                Some(reader) => self.render_document(reader.clone(), &entry),
                None => self.render_file(&entry),
            };
            
            if let Err(err) = result {
                println!("{}", err);
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
    pub created: Option<u64>,
    pub keywords: Vec<String>,
}

impl Default for DocumentInfo {
    fn default() -> DocumentInfo {
        DocumentInfo {
            title: "".into(),
            language: None,
            modified: None,
            created: None,
            keywords: Vec::new(),
        }
    }
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
