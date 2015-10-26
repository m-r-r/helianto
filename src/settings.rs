use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;
use rustc_serialize::{Decodable, Decoder};
use toml::Decoder as TomlDecoder;
use toml::{Parser, Value};
use super::Error;


#[derive(Clone, Debug)]
pub struct Settings {
    pub source_dir: PathBuf,
    pub output_dir: PathBuf,
    pub max_depth: usize,
    pub follow_links: bool,
    pub site_title: Option<String>,
    pub site_url: String,
    pub site_language: Option<String>,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            source_dir: PathBuf::from("."),
            output_dir: PathBuf::from("_output"),
            max_depth: ::std::usize::MAX,
            follow_links: false,
            site_title: None,
            site_url: String::from("/"),
            site_language: None,
        }
    }
}

impl Settings {
    pub fn from_file<P: AsRef<Path>>(path: &P) -> super::Result<Self> {
        let mut fd = try!(File::open(path.as_ref()));

        let mut content = String::new();
        try! {
            fd.read_to_string(&mut content)
        };

        let mut parser = Parser::new(content.as_ref());
        let toml = try! {
            parser.parse()
                  .map(|table| Value::Table(table))
                  .ok_or((path, parser))
        };

        let mut decoder = TomlDecoder::new(toml);
        let mut settings = try! {
            Settings::decode(&mut decoder)
                     .map_err(|e| Error::from((path, e)))
        };

        // Make paths relative to the settings file:
        settings.source_dir = path.as_ref().join(settings.source_dir);
        settings.output_dir = path.as_ref().join(settings.output_dir);
        Ok(settings)
    }
}

#[derive(Default, RustcDecodable)]
struct SiteSettings {
    pub title: Option<Option<String>>,
    pub url: Option<String>,
    pub language: Option<Option<String>>,
}

#[derive(Default, RustcDecodable)]
struct GeneratorSettings {
    pub source_dir: Option<PathBuf>,
    pub output_dir: Option<PathBuf>,
    pub max_depth: Option<usize>,
    pub follow_links: Option<bool>,
}



impl Decodable for Settings {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Settings, D::Error> {
        decoder.read_struct("root", 0, |decoder| {
            let site_settings = try!{
                decoder.read_struct_field::<Option<SiteSettings>, _>("site", 0, Decodable::decode)
            }.unwrap_or_default();
            
            let generator_settings = try!{
                decoder.read_struct_field::<Option<GeneratorSettings>, _>("generator", 0, Decodable::decode)
            }.unwrap_or_default();

            let default = Settings::default();
            Ok(Settings {
                site_title: site_settings.title.unwrap_or(default.site_title), 
                site_url: site_settings.url.unwrap_or(default.site_url), 
                site_language: site_settings.language.unwrap_or(default.site_language),
                source_dir: generator_settings.source_dir.unwrap_or(default.source_dir),
                output_dir: generator_settings.output_dir.unwrap_or(default.output_dir),
                max_depth: generator_settings.max_depth.unwrap_or(default.max_depth),
                follow_links: generator_settings.follow_links.unwrap_or(default.follow_links),
            })
        })
    }
}