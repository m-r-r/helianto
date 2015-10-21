use rustc_serialize::json::{Json, Object, ToJson};

#[derive(RustcEncodable, RustcDecodable)]
pub struct DocumentMetadata {
    pub title: String,
    pub language: Option<String>,
    pub modified: Option<u64>,
    pub created: Option<u64>,
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
            },
            value => value,
        }
    }
}

