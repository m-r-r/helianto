// Helianto -- static website generator
// Copyright © 2015-2016 Mickaël RAYBAUD-ROIG
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

extern crate chrono;
extern crate handlebars;
extern crate num;
extern crate pulldown_cmark;
extern crate regex;
extern crate serde;
extern crate toml;
extern crate walkdir;
#[macro_use]
extern crate log;

mod document;
mod error;
mod generators;
pub mod metadata;
pub mod readers;
mod settings;
mod site;
mod templates;
mod utils;

use handlebars::Handlebars;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use walkdir::{DirEntry, WalkDir};

pub use crate::document::{Document, DocumentContent, DocumentMetadata};
pub use crate::error::{Error, Result};
pub use crate::generators::Generator;
use crate::readers::Reader;
pub use crate::settings::Settings;
pub use crate::site::Site;
use crate::templates::Context;

pub struct Compiler {
    pub settings: Settings,
    pub site: Site,
    handlebars: Handlebars<'static>,
    readers: HashMap<String, Rc<dyn Reader>>,
    generators: Vec<Rc<dyn Generator>>,
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
        let Settings {
            ref source_dir,
            ref output_dir,
            ..
        } = self.settings;

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

    pub fn get_reader(&self, path: &Path) -> Option<Rc<dyn Reader>> {
        path.extension()
            .and_then(|extension| extension.to_str())
            .and_then(|extension_str| self.readers.get(extension_str))
            .cloned()
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

        let loader = &mut templates::Loader::new(&mut self.handlebars);
        loader.load_builtin_templates();

        let templates_dir = self.settings.source_dir.join("_layouts");

        if templates_dir.is_dir() {
            loader.load_templates(&templates_dir);
        }

        Ok(())
    }

    fn render_context(&self, context: Context, path: &Path) -> Result<()> {
        let output: String = self.handlebars.render("page.html", &context)
                .map_err(|err| Error::Render {
                    cause: Box::new(err)
                })?;

        let dest_file = self.settings.output_dir.join(&path);
        let dest_dir = dest_file.parent().unwrap();
        fs::create_dir_all(&dest_dir)
            .and_then(|_| {
                let mut fd = File::create(&dest_file)?;
                fd.write(output.as_ref())?;
                fd.sync_data()?;
                Ok(())
            })
            .map_err(|err| Error::Output {
                dest: dest_dir.into(),
                cause: Box::new(err),
            })
    }

    fn build_document(&mut self, reader: Rc<dyn Reader>, path: &Path) -> Result<()> {
        let (body, metadata) = reader.load(path)?;
        let dest = path
            .strip_prefix(&self.settings.source_dir)
            .map(|relpath| relpath.with_extension("html"))
            .unwrap();

        let document = Document {
            metadata: DocumentMetadata {
                url: dest.to_str().unwrap().into(),
                .. DocumentMetadata::from_raw(metadata.into_iter())?
            },
            content: DocumentContent::from(body),
        };

        debug!(
            "Rendering document {} in {} ...",
            path.display(),
            dest.display()
        );
        self.render_context(Context::new(&self.site, &document), &dest)
            .and_then(|_| {
                self.documents.insert(
                    dest.to_str().unwrap().into(),
                    Rc::new(document.metadata.clone()),
                );
                Ok(())
            })
    }

    fn copy_file(&mut self, path: &Path) -> Result<()> {
        let dest = path
            .strip_prefix(&self.settings.source_dir)
            .map(|relpath| self.settings.output_dir.join(relpath))
            .unwrap();
        let dest_dir = dest.parent().unwrap();

        fs::create_dir_all(&dest_dir)
            .and_then(|_| fs::copy(path, &dest))
            .and_then(|_| {
                debug!("Copying {} to {}", path.display(), dest.display());
                Ok(())
            })
            .map_err(|err| Error::Copy {
                from: path.into(),
                to: dest_dir.into(),
                cause: Box::new(err),
            })
    }

    fn run_generators(&mut self) -> Result<()> {
        let documents: Vec<Rc<DocumentMetadata>> = self.documents.values().cloned().collect();

        for generator in self.generators.iter() {
            let generated_docs = generator.generate(documents.as_ref())?;

            for generated_doc in generated_docs.iter() {
                if self.documents.contains_key(&generated_doc.metadata.url) {
                    continue;
                }
                trace!("Running generator");

                let dest = utils::remove_path_prefix(&generated_doc.metadata.url);
                if let Err(e) = self.render_context(Context::new(&self.site, generated_doc), &dest)
                {
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        self.check_settings()?;
        self.load_templates()?;

        let entries = WalkDir::new(&self.settings.source_dir)
            .min_depth(1)
            .max_depth(self.settings.max_depth)
            .follow_links(self.settings.follow_links)
            .into_iter();

        for entry in entries.filter_entry(filter_entry) {
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

        self.run_generators()?;

        Ok(())
    }
}

pub fn filter_entry(entry: &DirEntry) -> bool {
    let file_type = entry.file_type();

    if file_type.is_dir() || file_type.is_file() {
        utils::is_public(&entry.path())
    } else {
        false
    }
}
