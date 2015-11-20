use std::ascii::AsciiExt;
use rustc_serialize::json::{Json, Object, ToJson};
use super::{Result, Error};
use utils::{DateTime, FromRaw};
use metadata::{Date, Field, Keywords, Text, Value};

const TITLE_FIELD: &'static Field = &Text("title") as &Field;
const CREATED_FIELD: &'static Field = &Date("created") as &Field;
const MODIFIED_FIELD: &'static Field = &Date("modified") as &Field;
const KEYWORDS_FIELD:  &'static Field = &Keywords("keywords") as &Field;

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


        for (key, raw_value) in raw {
            let key = key.to_ascii_lowercase();
            
            match Self::read_raw_field(key.as_ref(), raw_value) {
               Ok(value) => {
                   match key.as_ref() {
                       "title" => { this.title = Option::unwrap_or_else(value.into(), String::new) },
                       "modified" => { this.modified = value.into() },
                       "created" => { this.created = value.into() },
                       "keywords" => { this.keywords = value.into() },
                       _ => (),
                   }
               }
                _ => continue,
            }
        }

        this
    }
    
    fn read_raw_field(key: &str, raw: &str) -> Result<Value> {

        let field = match key {
            "title" => TITLE_FIELD,
            "created" => CREATED_FIELD,
            "modified" => MODIFIED_FIELD,
            "keywords" => KEYWORDS_FIELD,
            other => return Err(
                Error::UnknownMetadataField {
                    name: key.into(),
                }
            )
        };
        
        field.from_raw(raw)
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
