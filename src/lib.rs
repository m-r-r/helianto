extern crate rustc_serialize;
extern crate walkdir;
extern crate handlebars;
#[macro_use(wrap)]
extern crate hoedown;
extern crate regex;
extern crate toml;

mod utils;
mod error;
mod document;
mod templates;
pub mod readers;
mod site;
mod settings;

use std::path::{Path, PathBuf};
use std::default::Default;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::rc::Rc;
use std::collections::HashMap;
use rustc_serialize::json::ToJson;
use walkdir::{WalkDir, WalkDirIterator};
use handlebars::Handlebars;

use utils::PathExt;
pub use error::{Error, Result};
use readers::Reader;
pub use document::{DocumentMetadata,Document};
pub use site::Site;
use templates::Context;
pub use settings::Settings;


pub struct Generator {
    pub settings: Settings,
    handlebars: Handlebars,
    readers: HashMap<String, Rc<Reader>>,
    site: Site,
    documents: Vec<(String, Rc<DocumentMetadata>)>,
}

impl Generator {
    pub fn new(settings: &Settings) -> Generator {
        let mut generator = Generator {
            settings: settings.clone(),
            readers: HashMap::new(),
            handlebars: Handlebars::new(),
            site: Site::new(settings),
            documents: Vec::new(),
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
        self.handlebars.clear_templates();

        // Default templates :
        self.handlebars.register_template_string("head.html", include_str!("templates/head.html.hbs").into()).unwrap();
        self.handlebars.register_template_string("page.html", include_str!("templates/page.html.hbs").into()).unwrap();
        self.handlebars.register_template_string("foot.html", include_str!("templates/foot.html.hbs").into()).unwrap();

        let template_dir = self.settings.source_dir.join("_layouts");

        if !template_dir.is_dir() {
            return Ok(());
        }


        let entries = WalkDir::new(template_dir)
                            .max_depth(7)
                            .into_iter()
                            .filter_entry(|entry| entry.file_type().is_dir() || utils::filter_template(entry))
                            .filter_map(|entry| entry.ok())
                            .map(|entry| PathBuf::from(entry.path()));


        for entry in entries {
            let template_name: String = match entry.with_extension("").clone().file_name().and_then(|n| n.to_str()) {
                Some(s) => s.into(),
                None => {
                    println!("Unable to load template file: {}: invalid file name", entry.display());
                    continue;
                }
            };

            let mut source = String::new();
            if let Err(e) = File::open(&entry).and_then(|mut fd| fd.read_to_string(&mut source)) {
                println!("Unable to load template file {}: {}", entry.display(), e);
                continue;
            }

            match self.handlebars.register_template_string(template_name.as_ref(), source) {
                Ok(_) => (),
                Err(e) => {
                    println!("Could not load tempalte file: {}: {}", entry.display(), e);
                }
            }
        }

        Ok(())
    }

    fn render_document(&mut self, reader: Rc<Reader>, path: &Path) -> Result<()> {
        let (body, metadata) = try! { reader.load(path) };
        let dest = path.relative_from_(&self.settings.source_dir)
                       .map(|relpath| relpath.with_extension("html"))
                       .unwrap();

        let document = Document {
            metadata: metadata.into_iter().collect(),
            content: body,
        };


        let payload = Context::new(&self.site, &document).to_json();
        let output: String = try! { self.handlebars.render("page.html", &payload)
                .map_err(|err| Error::Render {
                    cause: Box::new(err)
                })
        };

        let dest_file = self.settings.output_dir.join(&dest);
        let dest_dir = dest_file.parent().unwrap();
        fs::create_dir_all(&dest_dir)
            .and_then(|_| {
                let mut fd = try! { File::create(&dest_file) };
                try! { fd.write(output.as_ref()) };
                try! { fd.sync_data() };
                self.documents.push((dest.to_str().unwrap().into(), Rc::new(document.metadata.clone())));
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

    fn render_index(&mut self) -> Result<()> {
        use rustc_serialize::json::{Json,Object,ToJson};

        self.documents.sort_by(|first, second| {
            first.1.created.unwrap_or(0u64).cmp(&second.1.created.unwrap_or(0u64))
        });
        
        let documents: Vec<Json> = self.documents.iter().map(|&(ref url, ref metadata)| {
            match metadata.to_json() {
                Json::Object(mut obj) => {
                    obj.insert("url".into(), url.to_json());
                    Json::Object(obj)
                },
                v => v
            }
        }).collect();

        let payload = {
            let mut obj = Object::new();
            obj.insert("site".into(), self.site.to_json()); 
            obj.insert("pages".into(), documents.to_json()); 
            Json::Object(obj)
        };

        let output: String = try! { self.handlebars.render("page.html", &payload)
                .map_err(|err| Error::Render {
                    cause: Box::new(err)
                })
        };

        let dest_file = self.settings.output_dir.join("index.html");
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

    pub fn run(&mut self) -> Result<()> {
        self.check_settings();
        try!{self.load_templates()};

        let entries = WalkDir::new(&self.settings.source_dir)
                          .max_depth(self.settings.max_depth)
                          .follow_links(self.settings.follow_links)
                          .into_iter();

        for entry in entries.filter_entry(|entry| utils::valid_filename(entry.path())) {
            let entry = match entry {
                Err(_) => continue,
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

        try! { self.render_index() }

        Ok(())
    }
}
