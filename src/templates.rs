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


use std::path::{Path};
use rustc_serialize::json::{Json, Object, ToJson};
use handlebars::{self, Handlebars, Helper, JsonRender, RenderContext, RenderError};
use chrono::DateTime;
use walkdir::{WalkDir, WalkDirIterator, DirEntry};
use super::{Document, Site};

pub struct Context<'a> {
    pub site: &'a Site,
    pub document: &'a Document,
}

impl<'a> Context<'a> {
    pub fn new<'b>(site: &'b Site, document: &'b Document) -> Context<'b> {
        Context {
            site: site,
            document: document,
        }
    }
}

impl<'a> ToJson for Context<'a> {
    fn to_json(&self) -> Json {
        let mut obj = Object::new();
        obj.insert("site".into(), self.site.to_json());
        obj.insert("page".into(), self.document.to_json());
        Json::Object(obj)
    }
}


fn date_helper(c: &handlebars::Context,
                   h: &Helper,
                   _: &Handlebars,
                   rc: &mut RenderContext)
                   -> Result<(), RenderError> {
    let value_param = try!(h.param(0).ok_or_else(|| {
        RenderError { desc: "Param not found for helper \"date\"".into() }
    }));
    let format_param = try! { h.hash_get("format").ok_or(RenderError {
            desc: "Parameter \"format\" missing for helper \"date\"".into()
        }).and_then(|json| json.as_string().ok_or(RenderError {
            desc: "Parameter \"format\" must be a string".into()
        }))
    };

    let argument = if value_param.starts_with("@") {
                       rc.get_local_var(value_param)
                   } else {
                       c.navigate(rc.get_path(), value_param)
                   }
                   .clone();

    let value = if argument.is_null() {
        return Ok(());
    } else {
        argument.render()
    };

    let date = try! {
        DateTime::parse_from_rfc3339(value.as_ref()).map_err(|_| RenderError {
            desc: "Parameter #1 is not a valid date".into()
        })
    };

    let _ = try! {
        write!(rc.writer, "{}", date.format(format_param.as_ref()))
    };
    Ok(())
}


fn join_helper(c: &handlebars::Context,
                   h: &Helper,
                   _: &Handlebars,
                   rc: &mut RenderContext)
                   -> Result<(), RenderError> {
    let value_param = try!(h.param(0).ok_or_else(|| {
        RenderError { desc: "Param not found for helper \"join\"".into() }
    }));

    let separator = h.hash_get("separator")
        .and_then(|json| json.as_string())
        .unwrap_or(", ");

    let argument = if value_param.starts_with("@") {
                       rc.get_local_var(value_param)
                   } else {
                       c.navigate(rc.get_path(), value_param)
                   }
                   .clone();

    if let Some(items) = argument.as_array() {
        let result: Vec<String> = items.iter().map(|item| item.render()).collect();
        let _ = try!(write!(rc.writer, "{}", result.join(separator)));
    }

    Ok(())
}

pub fn register_helpers(handlebars: &mut Handlebars) {
    handlebars.register_helper("date", Box::new(date_helper));
    handlebars.register_helper("join", Box::new(join_helper));
}




pub struct Loader<'r> {
    pub registry: &'r mut Handlebars,
}

impl<'r> Loader<'r> {
    pub fn new(registry: &'r mut Handlebars) -> Self {
        Loader {
            registry: registry,
        }
    }

    pub fn load_builtin_templates(&mut self) {
        // Default templates :
        self.registry
            .register_template_string("head.html", include_str!("templates/head.html.hbs").into())
            .unwrap();
        self.registry
            .register_template_string("page.html", include_str!("templates/page.html.hbs").into())
            .unwrap();
        self.registry
            .register_template_string("foot.html", include_str!("templates/foot.html.hbs").into())
            .unwrap();
    }

    pub fn load_templates(&mut self, templates_dir: &Path) {
        let iter = WalkDir::new(templates_dir)
            .into_iter()
            .filter_entry(|e| filter_templates(e));

        for result in iter {
            let path = match result {
                Ok(ref entry) => entry.path(),
                Err(e) => {
                    error!("{}", e);
                    continue;
                }
            };

            let template_name = match template_name(templates_dir, path) {
                Some(path) => path,
                None => {
                    error!("Could not load template {}: invalid path", path.display());
                    continue;
                }
            };

            match self.registry.register_template_file(template_name.as_ref(), path) {
                Ok(()) => (),
                Err(e) => {
                    error!("Could not load template {}: {}", path.display(), e);
                    continue;
                }
            }
        }
    }
}

fn template_name(templates_dir: &Path, template_path: &Path) -> Option<String> {
    template_path.with_extension("")
        .strip_prefix(templates_dir)
        .or_else(|e| {
            debug!("Path::strip_prefix() -> {}", e);
            Err(e)
        })
        .ok()
        .and_then(|p| p.to_str())
        .map(|s| s.into())
}

/// Tests wether a directory entry is an Handlebar template.
fn filter_templates(entry: &DirEntry) -> bool {
    let file_type = entry.file_type();
    if file_type.is_dir() {
        true
    } else if file_type.is_file() {
        entry.path().extension() == Some("hbs".as_ref())
    } else {
        false
    }
}

