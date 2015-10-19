use std::path::Path;
use std::fs::File;
use hoedown::{Markdown, Render, Wrapper, Html};
use super::{Reader, Metadata};
use super::super::{Settings, Error, Result};

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

        Ok((body, ()))
    }
}


struct HtmlRender {
    base: Html,
}

impl HtmlRender {
    fn new() -> HtmlRender {
        use hoedown::renderer::html::Flags;

        HtmlRender { base: Html::new(Flags::empty(), 6) }
    }
}

impl Wrapper for HtmlRender {
    type Base = Html;

    fn base(&mut self) -> &mut Html {
        &mut self.base
    }
}

wrap!(HtmlRender);
