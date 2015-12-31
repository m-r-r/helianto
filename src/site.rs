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


use std::default::Default;
use rustc_serialize::json::{Json, Object, ToJson};
use super::Settings;

#[derive(Debug, RustcEncodable, RustcDecodable)]
pub struct Site {
    pub title: String,
    pub language: Option<String>,
    pub url: String,
}

impl ToJson for Site {
    fn to_json(&self) -> Json {
        let mut obj = Object::new();
        obj.insert("title".into(), self.title.to_json());
        obj.insert("language".into(), self.language.to_json());
        obj.insert("url".into(), self.url.to_json());
        Json::Object(obj)
    }
}

impl Default for Site {
    fn default() -> Site {
        Site {
            title: "Untitled website".into(),
            language: None,
            url: "/".into(),
        }
    }
}

impl Site {
    pub fn new(setting: &Settings) -> Site {
        let mut site = Site::default();
        if let Some(ref title) = setting.site_title {
            site.title = title.clone();
        }
        site.url = setting.site_url.clone();
        if let Some(ref language) = setting.site_language {
            site.language = Some(language.clone());
        }
        site
    }
}
