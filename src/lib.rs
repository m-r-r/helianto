extern crate rustc_serialize;
extern crate walkdir;
extern crate handlebars;
#[macro_use(wrap)]
extern crate hoedown;
extern crate regex;
extern crate toml;
extern crate chrono;
extern crate num;
#[macro_use]
extern crate log;

mod utils;
mod error;
mod document;
mod templates;
pub mod readers;
pub mod metadata;
mod site;
mod settings;
mod generators;

use std::path::{Path, PathBuf};
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
pub use document::{Document, DocumentMetadata, DocumentContent};
pub use site::Site;
use templates::Context;
pub use settings::Settings;
pub use generators::Generator;


pub struct Compiler {
    pub settings: Settings,
    pub site: Site,
    handlebars: Handlebars,
    readers: HashMap<String, Rc<Reader>>,
    generators: Vec<Rc<Generator>>,
    documents: HashMap<String, Rc<DocumentMetadata>>,
}

impl Compiler {
    pub fn new(settings: &Settings) -> Compiler {
        let mut compiler = Compiler {
            settings: settings.clone(),
            readers: HashMap::new(),
            handlebars: Handlebars::new(),
            site: Site::new(settings),
            documents: HashMap::new(),
            generators: Vec::new(),
        };
        compiler.add_reader::<readers::MarkdownReader>();
        compiler.add_generator::<generators::IndexGenerator>();
        compiler
    }


    fn check_settings(&self) -> Result<()> {
        let Settings { ref source_dir, ref output_dir, .. } = self.settings;

        if !source_dir.is_dir() {
            return Err(Error::Settings {
                message: format!("{} must be an existing directory", source_dir.display()),
            });
        }

        if output_dir.exists() && !output_dir.is_dir() {
            return Err(Error::Settings {
                message: format!("{} must be a directory", output_dir.display()),
            });
        }

        Ok(())
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

    pub fn add_generator<T: Generator + 'static>(&mut self) {
        self.generators.push(Rc::new(T::new()));
    }

    fn load_templates(&mut self) -> Result<()> {
        self.handlebars.clear_templates();
        templates::register_helpers(&mut self.handlebars);

        // Default templates :
        self.handlebars
            .register_template_string("head.html", include_str!("templates/head.html.hbs").into())
            .unwrap();
        self.handlebars
            .register_template_string("page.html", include_str!("templates/page.html.hbs").into())
            .unwrap();
        self.handlebars
            .register_template_string("foot.html", include_str!("templates/foot.html.hbs").into())
            .unwrap();

        let template_dir = self.settings.source_dir.join("_layouts");

        if !template_dir.is_dir() {
            return Ok(());
        }


        let entries = WalkDir::new(template_dir)
                          .max_depth(7)
                          .into_iter()
                          .filter_entry(|entry| {
                              entry.file_type().is_dir() || utils::filter_template(entry)
                          })
                          .filter_map(|entry| match entry {
                              Ok(ref path) if path.file_type().is_file() => Some(path.clone()),
                              _ => None
                          })
                          .map(|entry| PathBuf::from(entry.path()));


        for entry in entries {
            let template_name: String = match entry.with_extension("")
                                                   .clone()
                                                   .file_name()
                                                   .and_then(|n| n.to_str()) {
                Some(s) => s.into(),
                None => {
                    error!("Unable to load template file: {}: invalid file name",
                             entry.display());
                    continue;
                }
            };

            let mut source = String::new();
            if let Err(e) = File::open(&entry).and_then(|mut fd| fd.read_to_string(&mut source)) {
                error!("Unable to read template file {}: {}", entry.display(), e);
                continue;
            }

            match self.handlebars.register_template_string(template_name.as_ref(), source) {
                Ok(_) => (),
                Err(e) => {
                    error!("Could not load tempalte file: {}: {}", entry.display(), e);
                }
            }
        }

        Ok(())
    }

    fn render_context(&self, context: Context, path: &Path) -> Result<()> {
        let payload = context.to_json();
        let output: String = try! { self.handlebars.render("page.html", &payload)
                .map_err(|err| Error::Render {
                    cause: Box::new(err)
                })
        };

        let dest_file = self.settings.output_dir.join(&path);
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
    
    fn build_document(&mut self, reader: Rc<Reader>, path: &Path) -> Result<()> {
        let (body, metadata) = try! { reader.load(path) };
        let dest = path.relative_from_(&self.settings.source_dir)
                       .map(|relpath| relpath.with_extension("html"))
                       .unwrap();

        let document = Document {
            metadata: DocumentMetadata {
                url: dest.to_str().unwrap().into(),
                .. try! { DocumentMetadata::from_raw(metadata.into_iter()) }
            },
            content: DocumentContent::from(body),
        };


        debug!("Rendering document {} in {} ...", path.display(), dest.display());
        self.render_context(Context::new(&self.site, &document), &dest)
            .and_then(|_| {
                self.documents.insert(dest.to_str().unwrap().into(),
                                     Rc::new(document.metadata.clone()));
                Ok(())
            })
    }

    fn copy_file(&mut self, path: &Path) -> Result<()> {
        let dest = path.relative_from_(&self.settings.source_dir)
                       .map(|relpath| self.settings.output_dir.join(relpath))
                       .unwrap();
        let dest_dir = dest.parent().unwrap();

        fs::create_dir_all(&dest_dir)
            .and_then(|_| fs::copy(path, &dest))
            .and_then(|_| {
                debug!("Copying {} to {}", path.display(), dest.display());
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

    fn run_generators(&mut self) -> Result<()> {
        let documents: Vec<Rc<DocumentMetadata>> = self.documents.values().map(|rc| rc.clone()).collect();

        for generator in self.generators.iter() {
            let generated_docs = try! { generator.generate(documents.as_ref()) };

            for generated_doc in generated_docs.iter() {
                if self.documents.contains_key(&generated_doc.metadata.url) {
                    continue;
                }
                trace!("Running generator");

                let dest = utils::remove_path_prefix(&generated_doc.metadata.url);
                if let Err(e) = self.render_context(Context::new(&self.site, generated_doc), &dest) {
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        try!{self.check_settings()};
        try!{self.load_templates()};

        let entries = WalkDir::new(&self.settings.source_dir)
                          .max_depth(self.settings.max_depth)
                          .follow_links(self.settings.follow_links)
                          .into_iter();

        for entry in entries.filter_entry(utils::filter_documents) {
            let entry = match entry {
                Err(_) => continue,
                Ok(e) => {
                    if e.file_type().is_dir() {
                        continue;
                    } else {
                        PathBuf::from(e.path())
                    }
                }
            };

            let result = match self.get_reader(&entry) {
                Some(reader) => self.build_document(reader.clone(), &entry),
                None => self.copy_file(&entry),
            };

            if let Err(err) = result {
                error!("{}", err);
            }
        }

        try!{self.run_generators()};


        Ok(())
    }
}
