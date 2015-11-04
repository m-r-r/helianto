use std::iter::FromIterator;
use std::ascii::AsciiExt;
use rustc_serialize::json::{Json, Object, ToJson};
use utils::DateTime;

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


impl FromIterator<(String,String)> for DocumentMetadata {
    fn from_iter<T>(iterator: T) -> Self
        where T: IntoIterator<Item = (String, String)>
    {
        let mut metadata = DocumentMetadata::default();
        for (key, value) in iterator {
            match key.to_ascii_lowercase().as_ref() {
                "title" => {
                    metadata.title = value;
                }
                "language" => {
                    metadata.language = Some(value);
                }
                "keywords" => {
                    metadata.keywords = value.split(",").map(|s| String::from(s)).collect();
                }
                "created" => {
                    metadata.created = DateTime::from_string(value.as_ref());
                }
                e => println!("Unknown metadata {}", e),
            }
        }
        metadata
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
