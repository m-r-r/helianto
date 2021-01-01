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

use crate::{Result, Settings};
use crate::readers::Reader;

use pulldown_cmark::{html, Options, Parser};

#[derive(Debug, Clone)]
pub struct MarkdownReader;

static EXTENSIONS: &[&str] = &["markdown", "md"];

impl Reader for MarkdownReader {
    fn new(_settings: &Settings) -> MarkdownReader {
        MarkdownReader
    }

    fn extensions() -> &'static [&'static str] {
        EXTENSIONS
    }

    fn read(&self, input: &str) -> Result<String> {
        let mut parser = Parser::new_ext(
            input,
            Options::ENABLE_TABLES | Options::ENABLE_FOOTNOTES,
        );
        let mut output = String::with_capacity(input.len() * (3 / 2));
        html::push_html(&mut output, &mut parser);
        Ok(output)
    }
}
