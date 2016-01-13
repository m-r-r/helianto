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


use std::iter::FromIterator;
use std::collections::HashMap;
use std::rc::Rc;
use std::ascii::AsciiExt;
use rustc_serialize::json::{Json, Object, ToJson};
use super::Result;
use utils::{DateTime, FromRaw};
use metadata::{Date, Field, Keywords};

const CREATED_FIELD: &'static Field = &Date("created") as &Field;
const MODIFIED_FIELD: &'static Field = &Date("modified") as &Field;
const KEYWORDS_FIELD: &'static Field = &Keywords("keywords") as &Field;

#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
pub struct DocumentMetadata {
    pub url: String,
    pub title: String,
    pub language: Option<String>,
    pub modified: Option<DateTime>,
    pub created: Option<DateTime>,
    pub keywords: Vec<String>,
}

impl Default for DocumentMetadata {
    fn default() -> DocumentMetadata {
        DocumentMetadata {
            url: "".into(),
            title: "".into(),
            language: None,
            modified: None,
            created: None,
            keywords: Vec::new(),
        }
    }
}


impl DocumentMetadata {
    pub fn from_raw<T>(raw: T) -> Result<DocumentMetadata>
        where T: Iterator<Item = (String, String)>
    {
        let mut metadata = DocumentMetadata::default();
        let mut raw_metadata: HashMap<String, String> = raw.map(|(key, value)| {
            (key.to_ascii_lowercase(), value)
        }).collect();

        if let Some(title) = raw_metadata.remove("title") {
            metadata.title = title.trim().into();
        }

        if let Some(language) = raw_metadata.remove("language") {
            metadata.language = Some(language.trim().into());
        }

        if let Some(keywords) = raw_metadata.remove("keywords") {
            metadata.keywords = try! { KEYWORDS_FIELD.from_raw(keywords.as_ref()) }.into();
        }

        if let Some(ref created) = raw_metadata.remove("created") {
            metadata.created = try! { CREATED_FIELD.from_raw(created) }.into();
        }

        if let Some(ref modified) = raw_metadata.remove("modified") {
            metadata.modified = try! { MODIFIED_FIELD.from_raw(modified) }.into();
        }

        Ok(metadata)
    }
}

#[test]
fn test_from_raw() {
    let raw_metadata: Vec<(String, String)> = vec! [
        ("title".into(), "Foo bar".into()),
        ("language".into(), "en".into()),
        ("created".into(), "2015-12-23T02:12:35+01:00".into()),
        ("keywords".into(), "foo, bar".into()),
    ];

    let metadata = DocumentMetadata::from_raw(raw_metadata.into_iter());
    assert!(metadata.is_ok());
    if let Ok(result) = metadata {
        assert_eq!(result.title, "Foo bar");
        assert_eq!(result.keywords.as_ref(), ["foo", "bar"]);
    }
}


impl ToJson for DocumentMetadata {
    fn to_json(&self) -> Json {
        let mut obj = Object::new();
        obj.insert("url".into(), self.url.to_json());
        obj.insert("title".into(), self.title.to_json());
        obj.insert("language".into(), self.language.to_json());
        obj.insert("modified".into(), self.modified.to_json());
        obj.insert("created".into(), self.created.to_json());
        obj.insert("keywords".into(), self.keywords.to_json());
        Json::Object(obj)
    }
}

pub enum DocumentContent {
    Text(String),
    Index(Vec<Rc<DocumentMetadata>>),
}

impl From<String> for DocumentContent {
    fn from(text: String) -> DocumentContent {
        DocumentContent::Text(text)
    }
}


impl FromIterator<Rc<DocumentMetadata>> for DocumentContent {
    fn from_iter<T>(documents: T) -> Self where T: IntoIterator<Item=Rc<DocumentMetadata>> {
        DocumentContent::Index(documents.into_iter().collect())
    }
}


pub struct Document {
    pub metadata: DocumentMetadata,
    pub content: DocumentContent,
}

impl Document {
    pub fn new(metadata: DocumentMetadata, content: DocumentContent) -> Document {
        Document { metadata: metadata, content: content }
    }
}


impl ToJson for Document {
    fn to_json(&self) -> Json {
        let mut obj = match self.metadata.to_json() {
            Json::Object(o) => o,
            _ => unreachable!("DocumentMetadata#to_json() must return a Json::Object."),
        };

        match self.content {
            DocumentContent::Text(ref content) => {
                obj.insert("content".into(), content.to_json());
            }
            DocumentContent::Index(ref documents) => {
                obj.insert("documents".into(), Json::Array(documents.iter()
                                                           .map(|doc| doc.to_json())
                                                           .collect()
                                                           ));
            }
        }

        Json::Object(obj)
    }
}

