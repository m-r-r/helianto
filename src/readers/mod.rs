use std::path::Path;
use super::Result;

mod markdown;
pub use self::markdown::MarkdownReader;

pub type Metadata = ();

pub trait Reader {
    fn extensions() -> &'static [&'static str] where Self: Sized;
    fn new(settings: &super::Settings) -> Self where Self: Sized;
    fn load(&self, path: &Path) -> Result<(String, Metadata)>;
}
