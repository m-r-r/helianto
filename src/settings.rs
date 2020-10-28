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

use super::utils::remove_leading_dot;
use super::{Error, Result};
use num::NumCast;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use toml::{self, Value};

#[derive(Clone, Debug)]
pub struct Settings {
    pub source_dir: PathBuf,
    pub output_dir: PathBuf,
    pub layouts_dir: PathBuf,
    pub max_depth: usize,
    pub follow_links: bool,
    pub site_title: String,
    pub site_url: String,
    pub site_language: Option<String>,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            source_dir: PathBuf::from("."),
            output_dir: PathBuf::from("_output"),
            layouts_dir: PathBuf::from("_layouts"),
            max_depth: ::std::usize::MAX,
            follow_links: false,
            site_title: String::from("Untitled"),
            site_url: String::from("/"),
            site_language: None,
        }
    }
}

impl Settings {
    pub fn with_working_directory(cwd: &Path) -> Settings {
        Settings {
            source_dir: cwd.join(".").into(),
            output_dir: cwd.join("_output").into(),
            layouts_dir: cwd.join("_layouts").into(),
            ..Settings::default()
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: &P) -> Result<Self> {
        let mut fd = try!(File::open(path.as_ref()));

        let mut content = String::new();
        try! {
            fd.read_to_string(&mut content)
        };

        let toml: Value =
            toml::de::from_str(content.as_str()).map_err(|err| Error::LoadSettings {
                path: path.as_ref().into(),
                cause: Box::new(err),
            })?;

        let parent_dir = path
            .as_ref()
            .parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));

        Settings::from_toml(&toml, &parent_dir)
    }

    fn from_toml(toml: &Value, cwd: &Path) -> Result<Self> {
        let mut settings = Settings::with_working_directory(cwd);

        macro_rules! get_value {
            ($key: expr) => {
                try!(read_value(toml, $key))
            };
        }

        macro_rules! set_field {
            ($field: expr, $value: expr) => {
                if let Some(tmp) = $value {
                    $field = tmp;
                }
            };
        }

        set_field!(settings.site_title, get_value!("site.title"));
        set_field!(settings.site_url, get_value!("site.url"));
        set_field!(
            settings.site_language,
            get_value!("site.language").map(Some)
        );

        set_field!(
            settings.output_dir,
            try!(read_directory(toml, "compiler.output_dir", cwd))
        );
        set_field!(
            settings.source_dir,
            try!(read_directory(toml, "compiler.source_dir", cwd))
        );
        set_field!(
            settings.layouts_dir,
            try!(read_directory(toml, "compiler.layouts_dir", cwd))
        );
        set_field!(settings.max_depth, get_value!("compiler.max_depth"));
        set_field!(settings.follow_links, get_value!("compiler.follow_links"));

        Ok(settings)
    }
}

trait FromToml: 'static + Sized {
    fn type_str() -> &'static str;
    fn from_toml(toml: &Value) -> Self;
}

impl FromToml for String {
    fn type_str() -> &'static str {
        "string"
    }

    fn from_toml(toml: &Value) -> String {
        toml.as_str().unwrap().into()
    }
}

impl FromToml for usize {
    fn type_str() -> &'static str {
        "integer"
    }

    fn from_toml(toml: &Value) -> usize {
        let number = toml.as_integer().unwrap();
        NumCast::from(number).expect("integer overflow")
    }
}

impl FromToml for bool {
    fn type_str() -> &'static str {
        "boolean"
    }

    fn from_toml(toml: &Value) -> bool {
        toml.as_bool().unwrap()
    }
}

fn read_value<T: FromToml>(toml: &Value, key: &str) -> Result<Option<T>> {
    if let Some(value) = toml.get(key) {
        if value.type_str() == T::type_str() {
            Ok(Some(FromToml::from_toml(value)))
        } else {
            Err(Error::Settings {
                message: format!(
                    "found a value of type `{}` instead of a value of type `{}` \
                                  for the key `{}`",
                    value.type_str(),
                    T::type_str(),
                    key
                ),
            })
        }
    } else {
        Ok(None)
    }
}

fn read_directory(toml: &Value, key: &str, cwd: &Path) -> Result<Option<PathBuf>> {
    let path: PathBuf = match try!(read_value::<String>(toml, key)) {
        None => return Ok(None),
        Some(v) => PathBuf::from(v),
    };

    Ok(Some(cwd.join(if path.starts_with(".") {
        remove_leading_dot(&path)
    } else {
        path
    })))
}
