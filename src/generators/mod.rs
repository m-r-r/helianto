use std::rc::Rc;
use super::Result;
use super::document::{Document, DocumentMetadata, DocumentContent};



pub trait Generator {
    fn new() -> Self where Self: Sized;
    fn generate(&self, docs: &[Rc<DocumentMetadata>]) -> Result<Vec<Rc<Document>>>;
}


