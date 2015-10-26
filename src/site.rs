use std::default::Default;
use rustc_serialize::json::{ToJson,Json,Object};
use super::{Settings};

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
