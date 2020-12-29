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

use super::{Document, Site};
use chrono::DateTime;
use handlebars::{
    self, Handlebars, Helper, HelperResult, JsonRender, Output, RenderContext, RenderError,
};
use serde::Serialize;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, Serialize)]
pub struct Context<'a> {
    pub site: &'a Site,
    #[serde(rename = "page")]
    pub document: &'a Document,
}

impl<'a> Context<'a> {
    pub fn new<'b>(site: &'b Site, document: &'b Document) -> Context<'b> {
        Context { site, document }
    }
}

fn date_helper(
    h: &Helper,
    _: &Handlebars,
    _c: &handlebars::Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let value = h
        .param(0)
        .map(|v| v.value())
        .ok_or(RenderError::new("Param not found for helper \"date\""))?
        .render();

    let format: String = h
        .hash_get("format")
        .ok_or(RenderError::new(
            "Parameter \"format\" missing for helper \"date\"",
        ))?
        .render();

    out.write(
        DateTime::parse_from_rfc3339(value.as_str())
            .map_err(|_| RenderError::new("Parameter #1 is not a valid date"))?
            .format(format.as_str())
            .to_string()
            .as_str(),
    )?;

    Ok(())
}

const DEFAULT_SEPARATOR: &str = ", ";

fn join_helper(
    h: &Helper,
    _: &Handlebars,
    _c: &handlebars::Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let value = h
        .param(0)
        .map(|v| v.value())
        .ok_or(RenderError::new("Param not found for helper \"join\""))?;

    let separator = h
        .hash_get("separator")
        .and_then(|pv| {
            if pv.is_value_missing() {
                None
            } else {
                Some(pv.render())
            }
        })
        .unwrap_or(String::from(DEFAULT_SEPARATOR));

    out.write(
        value
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or(&[])
            .iter()
            .map(|item| item.render())
            .collect::<Vec<String>>()
            .join(separator.as_str())
            .as_str(),
    )?;

    Ok(())
}

pub fn register_helpers(handlebars: &mut Handlebars<'static>) {
    handlebars.register_helper("date", Box::new(date_helper));
    handlebars.register_helper("join", Box::new(join_helper));
}

pub struct Loader<'r> {
    pub registry: &'r mut Handlebars<'static>,
}

impl<'r> Loader<'r> {
    pub fn new(registry: &'r mut Handlebars<'static>) -> Self {
        Loader { registry }
    }

    pub fn load_builtin_templates(&mut self) {
        // Default templates :
        self.registry
            .register_template_string("head.html", include_str!("templates/head.html.hbs"))
            .unwrap();
        self.registry
            .register_template_string("page.html", include_str!("templates/page.html.hbs"))
            .unwrap();
        self.registry
            .register_template_string("foot.html", include_str!("templates/foot.html.hbs"))
            .unwrap();
    }

    pub fn load_templates(&mut self, templates_dir: &Path) {
        let iter = WalkDir::new(templates_dir)
            .into_iter()
            .filter_entry(|e| filter_templates(e));

        for result in iter {
            let path = match result {
                Ok(ref entry) if entry.file_type().is_file() => entry.path(),
                Ok(_) => continue,
                Err(e) => {
                    error!("{}", e);
                    continue;
                }
            };
            debug!("{}", path.display());

            let template_name = match template_name(templates_dir, path) {
                Some(path) => path,
                None => {
                    error!("Could not load template {}: invalid path", path.display());
                    continue;
                }
            };

            match self
                .registry
                .register_template_file(template_name.as_ref(), path)
            {
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
    template_path
        .with_extension("")
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
