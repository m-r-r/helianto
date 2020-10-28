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

use super::super::{Error, Result, Settings};
use super::{Metadata, Reader};
use pulldown_cmark::{html, Event, Options, Parser, Tag};
use regex::Regex;
use std::ascii::AsciiExt;
use std::fs::File;
use std::io::Read;
use std::mem::replace;
use std::path::Path;

#[derive(Debug, Clone)]
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
        let mut input = String::new();
        try! {
            File::open(path)
                .and_then(|mut fd| fd.read_to_string(&mut input))
                .map_err(|err| Error::Reader {
                    path: path.into(),
                    cause: Box::new(err),
                }
            )
        };

        Ok(process_markdown(&input))
    }
}

#[derive(PartialEq, Eq, Debug)]
enum State {
    BeforeTitle,
    InsideTitle,
    BeforeMetadata,
    InsideMetadata,
    InsideBody,
}

struct MetadataExtractor<'a> {
    inner: Parser<'a>,
    regex: Regex,
    state: State,
    pub metadata: Metadata,
    buffer: Vec<Event<'a>>,
}

impl<'a> From<Parser<'a>> for MetadataExtractor<'a> {
    fn from(parser: Parser) -> MetadataExtractor {
        MetadataExtractor {
            inner: parser,
            regex: Regex::new(r"^[\w][\w\d_\x2D ]*\s*:").unwrap(),
            state: State::BeforeTitle,
            metadata: Metadata::new(),
            buffer: Vec::new(),
        }
    }
}

impl<'a> Iterator for MetadataExtractor<'a> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        use self::State::*;

        if self.state == State::InsideBody {
            if self.buffer.len() > 0 {
                return Some(self.buffer.remove(0));
            } else {
                return self.inner.next();
            }
        }

        let event = match self.inner.next() {
            Some(ev) => ev,
            None => return None,
        };

        match self.state {
            BeforeTitle => match event {
                Event::Start(Tag::Heading(_)) => {
                    self.state = State::InsideTitle;
                    self.next()
                }
                Event::Start(Tag::Paragraph) => {
                    self.state = State::InsideMetadata;
                    self.buffer.push(event);
                    self.next()
                }
                _ => {
                    self.state = State::InsideBody;
                    Some(event)
                }
            },
            InsideTitle => match event {
                Event::Text(_) => {
                    self.metadata.insert("title".into(), get_event_text(&event));
                    self.next()
                }
                Event::End(Tag::Heading(_)) => {
                    self.state = State::BeforeMetadata;
                    self.next()
                }
                _ => self.next(),
            },
            BeforeMetadata => match event {
                Event::Start(Tag::Paragraph) => {
                    self.state = State::InsideMetadata;
                    self.buffer.push(event);
                    self.next()
                }
                _ => {
                    self.state = State::InsideBody;
                    Some(event)
                }
            },
            InsideMetadata => match event {
                Event::Text(_) => {
                    if !self.regex.is_match(&get_event_text(&event)) {
                        self.state = State::InsideBody;
                    }
                    self.buffer.push(event);
                    self.next()
                }
                Event::End(Tag::Paragraph) => {
                    self.metadata.extend(
                        replace(&mut self.buffer, Vec::new())
                            .into_iter()
                            .filter_map(|event| {
                                if let Event::Text(text) = event {
                                    let (key, value) = split_pair(&text);
                                    Some((key.to_ascii_lowercase(), value))
                                } else {
                                    None
                                }
                            }),
                    );
                    self.state = State::InsideBody;
                    self.next()
                }
                Event::SoftBreak | Event::HardBreak => {
                    self.buffer.push(event);
                    self.next()
                }
                _ => {
                    self.buffer.push(event);
                    self.state = State::InsideBody;
                    self.next()
                }
            },
            InsideBody => unreachable!(),
        }
    }
}

fn split_pair<S: AsRef<str>>(input: &S) -> (String, String) {
    let mut split = input.as_ref().splitn(2, ':');
    let key: &str = split.next().unwrap_or("");
    let value: &str = split.next().unwrap_or("");
    (key.trim().into(), value.trim().into())
}

fn get_event_text<'a>(event: &Event<'a>) -> String {
    if let Event::Text(ref text) = *event {
        text.clone().to_string()
    } else {
        panic!()
    }
}

fn process_markdown<S: AsRef<str>>(input: &S) -> (String, Metadata) {
    let mut parser = MetadataExtractor::from(Parser::new_ext(
        input.as_ref(),
        Options::ENABLE_TABLES | Options::ENABLE_FOOTNOTES,
    ));
    let mut output = String::with_capacity(input.as_ref().len() * (3 / 2));
    html::push_html(&mut output, &mut parser);
    (output, parser.metadata.clone())
}

#[test]
fn extract_title() {
    let (output, metadata) = process_markdown(&"# Foo\nbar\nbaz");
    assert_eq!(metadata.get("title"), Some(&"Foo".into()));
    assert_eq!(output, "<p>bar\nbaz</p>\n");
}

#[test]
fn extract_metadata() {
    let (output, metadata) =
        process_markdown(&"# Foo\n\nBar: baz:quux\nFoo bar: qux baz\n\nfoo: bar");
    assert_eq!(metadata.get("title"), Some(&"Foo".into()));
    assert_eq!(metadata.get("bar"), Some(&"baz:quux".into()));
    assert_eq!(metadata.get("foo bar"), Some(&"qux baz".into()));
    assert_eq!(output, "<p>foo: bar</p>\n");
}

#[test]
fn can_skip_to_metadata() {
    let (output, metadata) = process_markdown(&"\n\n\nBar: baz:quux\nFoo bar: qux baz\n\nfoo: bar");
    assert_eq!(metadata.get("title"), None);
    assert_eq!(metadata.get("bar"), Some(&"baz:quux".into()));
    assert_eq!(metadata.get("foo bar"), Some(&"qux baz".into()));
    assert_eq!(output, "<p>foo: bar</p>\n");
}

#[test]
fn can_skip_to_body() {
    let (output, metadata) =
        process_markdown(&"\n\n\nBar: baz:quux\nFoo bar: qux baz  \nlol\n\nfoo: bar");
    assert_eq!(metadata.get("title"), None);
    assert_eq!(metadata.get("bar"), None);
    assert_eq!(metadata.get("foo bar"), None);
    assert_eq!(
        output,
        "<p>Bar: baz:quux\nFoo bar: qux baz<br />\nlol</p>\n<p>foo: bar</p>\n"
    );
}

#[test]
fn can_skip_metadata() {
    let (output, metadata) =
        process_markdown(&"# Title\n\n\nBar: baz:quux\nFoo bar: qux baz  \nlol\n\nfoo: bar");
    assert_eq!(metadata.get("title"), Some(&String::from("Title")));
    assert_eq!(metadata.get("bar"), None);
    assert_eq!(metadata.get("foo bar"), None);
    assert_eq!(
        output,
        "<p>Bar: baz:quux\nFoo bar: qux baz<br />\nlol</p>\n<p>foo: bar</p>\n"
    );
}
