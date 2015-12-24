
use std::rc::Rc;
use std::collections::HashMap;
use std::path::PathBuf;
use super::super::{Result, Document, DocumentMetadata, DocumentContent};



pub struct IndexGenerator;

impl super::Generator for IndexGenerator {
    fn new() -> IndexGenerator { IndexGenerator }

    fn generate(&self, docs: &[Rc<DocumentMetadata>]) -> Result<Vec<Rc<Document>>> {
        let mut indexes: HashMap<String, Vec<Rc<DocumentMetadata>>> = HashMap::new();

        for doc in docs.iter() {
            let dir: String = PathBuf::from(&doc.url)
                .parent()
                .and_then(|p| p.to_str())
                .unwrap_or("")
                .into();

            println!("Adding page {} to the index {}...", doc.url, dir);
            let documents = indexes.entry(dir).or_insert_with(Vec::new);
            documents.push(doc.clone());
        }


        Ok(indexes.into_iter()
                  .map(|(url, docs)| {

                      let meta = DocumentMetadata {
                          url: format!("{}/index.html", url),
                          title: format!("Index of {}", url),
                          ..DocumentMetadata::default()
                      };
                      let content = DocumentContent::Index(docs);

                      Rc::new(Document::new(meta, content))
                  })
                  .collect())
    }
}
