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


use std::path::Path;
use std::fs::File;
use regex::Regex;
use hoedown::{Buffer, Html, Markdown, Render, Wrapper};
use super::{Metadata, Reader};
use super::super::{Error, Result, Settings};

#[derive(Debug)]
pub struct MarkdownReader;

static EXTENSIONS: &'static [&'static str] = &["markdown", "md", "mkd", "mdown"];

impl Reader for MarkdownReader {
    fn new(_settings: &Settings) -> MarkdownReader {
        MarkdownReader
    }

    fn extensions() -> &'static [&'static str] {
        EXTENSIONS
    }

    fn load(&self, path: &Path) -> Result<(String, Metadata)> {
        let fd = try! { File::open(path) };
        let mut renderer = HtmlRender::new();
        let input = Markdown::read_from(fd);
        let body = String::from(try! { renderer.render(&input).to_str()
            .map_err(|err| Error::Reader {
                    path: path.into(),
                    cause: Box::new(err),
                }
            )
        });

        Ok((body, renderer.into()))
    }
}


struct HtmlRender {
    base: Html,
    before_header: bool,
    before_metadata: bool,
    metadata: Metadata,
    metadata_regex: Regex,
}

impl HtmlRender {
    fn new() -> HtmlRender {
        use hoedown::renderer::html::Flags;

        HtmlRender {
            base: Html::new(Flags::empty(), 6),
            before_header: true,
            before_metadata: true,
            metadata: Metadata::new(),
            metadata_regex: Regex::new(r"(?m)\A[ \t\n]*((?:[ \t]*[^ \t\\:]+[ \t]*:[^\n]*[\n]*$)+)[ \t\n]*\z").unwrap(),
        }
    }

    fn read_metadata(&mut self, buffer: &Buffer) {
        if let Ok(metadata_str) = buffer.to_str() {
            for line in metadata_str.lines() {
                let parts: Vec<&str> = line.splitn(2, ":").collect();
                if let (Some(key), Some(value)) = (parts.get(0), parts.get(1)) {
                    self.metadata.insert(String::from(key.trim()), String::from(value.trim()));
                }
            }
        }
    }
}

impl Wrapper for HtmlRender {
    type Base = Html;

    fn base(&mut self) -> &mut Html {
        &mut self.base
    }

    fn header(&mut self, ob: &mut Buffer, content: &Buffer, level: i32) {
        if self.before_header {
            if let Ok(title) = content.to_str().map(|s| s.trim()) {
                self.metadata.insert("title".into(), title.into());
                self.before_header = false;
            }
        } else {
            self.base().header(ob, content, level);
        }
    }

    fn paragraph(&mut self, ob: &mut Buffer, content: &Buffer) {
        let is_metadata = content.to_str()
                                 .ok()
                                 .map(|s| self.metadata_regex.is_match(s))
                                 .unwrap_or(false);
        if !self.before_header && self.before_metadata && is_metadata {
            self.read_metadata(content);
        } else {
            self.base().paragraph(ob, content);
        }
    }
}

impl Into<Metadata> for HtmlRender {
    fn into(self) -> Metadata {
        self.metadata
    }
}

wrap!(HtmlRender);
