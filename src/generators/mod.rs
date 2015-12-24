use std::rc::Rc;
use super::Result;
use super::document::{Document, DocumentMetadata, DocumentContent};



pub trait Generator {
    fn new() -> Self where Self: Sized;
    fn generate(&self, docs: &[Rc<DocumentMetadata>]) -> Result<Vec<Rc<Document>>>;
}

/*
struct IndexGenerator;

impl Generator for IndexGenerator {
    fn generate(&self, docs: &[Rc<DocumentMetadata>]) -> Result<Vec<Rc<Entry>>> {
        let mut indexes: HashMap<String, Vec<Rc<DocumentMetadata>>> = HashMap::new();

        for doc in docs.iter() {
            let dir: String = doc.url.rsplitn(1, '/').next().unwrap_or("").into();
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

                      Rc::new(Entry::DocumentIndex(DocumentIndex {
                          metadata: meta,
                          documents: docs,
                      }))
                  })
                  .collect())
    }
}
*/
