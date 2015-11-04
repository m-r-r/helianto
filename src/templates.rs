use rustc_serialize::json::{Json, Object, ToJson};
use super::{Document, Site};
use handlebars::{self, Helper, Handlebars, RenderContext, RenderError, JsonRender};
use chrono::{DateTime};

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


#[derive(Clone, Copy)]
pub struct DateHelper;

pub fn date_helper(c: &handlebars::Context, h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
    let value_param = try!(h.param(0).ok_or_else(|| RenderError { desc: "Param not found for helper \"date\"" }));
    let format_param = try! { h.hash_get("format").ok_or(RenderError {
            desc: "Parameter \"format\" missing for helper \"date\""
        }).and_then(|json| json.as_string().ok_or(RenderError {
            desc: "Parameter \"format\" must be a string"
        }))
    };

    let argument = if value_param.starts_with("@") {
        rc.get_local_var(value_param)
    } else {
        c.navigate(rc.get_path(), value_param)
    }.clone();

    let value = if argument.is_null() {
        return Ok(())
    } else {
        argument.render()
    };

    let date = try! {
        DateTime::parse_from_rfc3339(value.as_ref()).map_err(|_| RenderError {
            desc: "Parameter #1 is not a valid date"
        })
    };

    let _ = try! {
        write!(rc.writer, "{}", date.format(format_param.as_ref()))
    };
    Ok(())
}
