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


use rustc_serialize::json::{Json, Object, ToJson};
use super::{Document, Site};
use handlebars::{self, Handlebars, Helper, JsonRender, RenderContext, RenderError};
use chrono::DateTime;

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


fn date_helper(c: &handlebars::Context,
                   h: &Helper,
                   _: &Handlebars,
                   rc: &mut RenderContext)
                   -> Result<(), RenderError> {
    let value_param = try!(h.param(0).ok_or_else(|| {
        RenderError { desc: "Param not found for helper \"date\"" }
    }));
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
                   }
                   .clone();

    let value = if argument.is_null() {
        return Ok(());
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


fn join_helper(c: &handlebars::Context,
                   h: &Helper,
                   _: &Handlebars,
                   rc: &mut RenderContext)
                   -> Result<(), RenderError> {
    let value_param = try!(h.param(0).ok_or_else(|| {
        RenderError { desc: "Param not found for helper \"join\"" }
    }));

    let separator = h.hash_get("separator")
        .and_then(|json| json.as_string())
        .unwrap_or(", ");

    let argument = if value_param.starts_with("@") {
                       rc.get_local_var(value_param)
                   } else {
                       c.navigate(rc.get_path(), value_param)
                   }
                   .clone();

    if let Some(items) = argument.as_array() {
        let result: Vec<String> = items.iter().map(|item| item.render()).collect();
        let _ = try!(write!(rc.writer, "{}", result.join(separator)));
    }

    Ok(())
}

pub fn register_helpers(handlebars: &mut Handlebars) {
    handlebars.register_helper("date", Box::new(date_helper));
    handlebars.register_helper("join", Box::new(join_helper));
}
