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

use crate::Result;
use crate::metadata::{DateTime, Value};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::iter::FromIterator;
use std::rc::Rc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub url: String,
    pub title: String,
    pub language: Option<String>,
    pub modified: Option<DateTime>,
    pub created: Option<DateTime>,
    pub keywords: Vec<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
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
            extra: HashMap::new(),
        }
    }
}

#[test]
fn test_decode_metadata() {
    use serde_yaml::from_str;
    let doc = r#"
url: "/foo/bar"
title: "Quux"
language: fr
created: 2020-12-30 14:47:30+01:00
keywords: 
    - foo
    - bar
extra:
    - foo
    - bar
quux: 42"#;

    let doc_metadata = from_str::<DocumentMetadata>(doc).unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DocumentContent {
    Text {
        content: String,
    },
    Index {
        documents: Vec<Rc<DocumentMetadata>>,
    },
}

impl From<String> for DocumentContent {
    fn from(text: String) -> DocumentContent {
        DocumentContent::Text { content: text }
    }
}

impl FromIterator<Rc<DocumentMetadata>> for DocumentContent {
    fn from_iter<T>(documents: T) -> Self
    where
        T: IntoIterator<Item = Rc<DocumentMetadata>>,
    {
        DocumentContent::Index {
            documents: documents.into_iter().collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    #[serde(flatten)]
    pub metadata: DocumentMetadata,
    #[serde(flatten)]
    pub content: DocumentContent,
}

impl Document {
    pub fn new(metadata: DocumentMetadata, content: DocumentContent) -> Document {
        Document { metadata, content }
    }
}
