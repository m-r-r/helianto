use std::ascii::AsciiExt;
use rustc_serialize::json::{Json, Object, ToJson};
use utils::{DateTime, FromRaw};
use metadata::{Date, Field, Keywords, Text};
use std::any::Any;


#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
pub struct DocumentMetadata {
    pub title: String,
    pub language: Option<String>,
    pub modified: Option<DateTime>,
    pub created: Option<DateTime>,
    pub keywords: Vec<String>,
}

impl Default for DocumentMetadata {
    fn default() -> DocumentMetadata {
        DocumentMetadata {
            title: "".into(),
            language: None,
            modified: None,
            created: None,
            keywords: Vec::new(),
        }
    }
}


impl DocumentMetadata {
    pub fn from_raw<'l, T>(raw: T) -> Self
        where T: Iterator<Item = (&'l String, &'l String)>
    {
        let mut this = DocumentMetadata::default();

        macro_rules! set_field (
            ($map: expr, $name: ident  :  $field: path, $raw: expr) => (
                match $field(stringify![title]).from_raw($raw.as_ref()) {
                    Ok(value) => {
                        $map.$name = Option::unwrap_or(value.into(), $map.$name);
                    },
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
            )
        );

        for (key, value) in raw {
            match key.to_ascii_lowercase().as_ref() {
                "title" => set_field!(this, title: Text, value),
                _ => continue,
            }
        }

        this
    }
}


impl ToJson for DocumentMetadata {
    fn to_json(&self) -> Json {
        let mut obj = Object::new();
        obj.insert("title".into(), self.title.to_json());
        obj.insert("language".into(), self.language.to_json());
        obj.insert("modified".into(), self.modified.to_json());
        obj.insert("created".into(), self.created.to_json());
        obj.insert("keywords".into(), self.keywords.to_json());
        Json::Object(obj)
    }
}

pub struct Document {
    pub metadata: DocumentMetadata,
    pub content: String,
}


impl ToJson for Document {
    fn to_json(&self) -> Json {
        match self.metadata.to_json() {
            Json::Object(mut obj) => {
                obj.insert("content".into(), self.content.to_json());
                Json::Object(obj)
            }
            value => value,
        }
    }
}
