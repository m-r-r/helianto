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

use super::super::{Document, DocumentContent, DocumentMetadata, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

pub struct IndexGenerator;

impl super::Generator for IndexGenerator {
    fn new() -> IndexGenerator {
        IndexGenerator
    }

    fn generate(&self, docs: &[Rc<DocumentMetadata>]) -> Result<Vec<Rc<Document>>> {
        let mut indexes: HashMap<String, Vec<Rc<DocumentMetadata>>> = HashMap::new();

        for doc in docs.iter() {
            let mut path: PathBuf = PathBuf::from(&doc.url);

            while let Some(parent) = path.clone().parent() {
                let index_url = match parent.to_str() {
                    Some(string) => string.into(),
                    None => continue,
                };

                let documents = indexes.entry(index_url).or_insert_with(Vec::new);
                documents.push(doc.clone());

                path = parent.into();
            }
        }

        Ok(indexes
            .into_iter()
            .map(|(url, mut docs)| {
                let meta = DocumentMetadata {
                    url: format!("{}/index.html", url),
                    title: format!("Index of {}", if url != "" { url } else { "/".into() }),
                    ..DocumentMetadata::default()
                };

                docs.sort_by(|b, a| a.created.cmp(&b.created));
                let content = DocumentContent::Index { documents: docs };

                Rc::new(Document::new(meta, content))
            })
            .collect())
    }
}
