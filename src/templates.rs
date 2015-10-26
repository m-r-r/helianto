use rustc_serialize::json::{Json, Object, ToJson};
use super::{Document, Site};

pub struct Context<'a> {
    pub site: &'a Site,
    pub document: &'a Document,
}

impl<'a> Context<'a> {
    pub fn new<'b>(site: &'b Site, document: &'b Document) -> Context<'b> {
        Context {
            site: site,
            document: document,
        }
    }
}

impl<'a> ToJson for Context<'a> {
    fn to_json(&self) -> Json {
        let mut obj = Object::new();
        obj.insert("site".into(), self.site.to_json());
        obj.insert("page".into(), self.document.to_json());
        Json::Object(obj)
    }
}
