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


use std::result;
use std::path::{PathBuf, Path, Component};
use walkdir::DirEntry;
use chrono::{self, FixedOffset};
use rustc_serialize::{Decodable, Decoder, Encodable, Encoder};
use rustc_serialize::json::{Json, ToJson};
use std::ascii::AsciiExt;
use std::ffi::OsStr;

static INVALID_CHARS: &'static str = "._~#$";

static TEMPLATE_EXTENSION: &'static str = ".hbs";


/// Check if a filename is valid
///
/// A valid filename must not start with `.`, `_`, `~`, `#` or `$` and must be at least one
/// character long.
fn valid_file_name<S: AsRef<OsStr> + ?Sized>(file_name: &S) -> bool {
    file_name.as_ref()
             .to_str()
             .and_then(|s| s.chars().next())
             .map(|first_char| !INVALID_CHARS.contains(first_char))
             .unwrap_or(false)
}


pub fn filter_source_entry(source_dir: &Path, entry: &DirEntry) -> bool {
    let path = entry.path();

    if ! path.starts_with(source_dir) {
        return false
    }

    let rel_path = match path.strip_prefix(source_dir) {
        Ok(value) => value,
        Err(_) => return false,
    };

    if rel_path.components().all(|c| valid_file_name(c.as_os_str())) {
        let file_type = entry.file_type();
        file_type.is_dir() || file_type.is_file()
    } else {
        false
    }
}


/// Remove the prefixes from a path
pub fn remove_path_prefix<S: AsRef<Path>>(path: S) -> PathBuf {
    path.as_ref()
        .components()
        .filter_map(|component| {
            if let Component::Normal(part) = component {
                Some(part)
            } else {
                None
            }
        })
        .collect()
}


/// Remove the dot at the begining of a path
pub fn remove_leading_dot<S: AsRef<Path>>(path: S) -> PathBuf {
    let path_ref = path.as_ref();
    if path_ref.starts_with(".") {
        path_ref.components()
            .skip(1)
            .map(Component::as_os_str)
            .collect()
    } else {
        path_ref.into()
    }
}

#[test]
fn test_remove_leading_dot() {
    const PATHS: &'static [(&'static str, &'static str)] = &[
        ("/foo/bar/baz", "/foo/bar/baz"),
        ("foo/bar/baz", "foo/bar/baz"),
        ("./foo/bar/baz", "foo/bar/baz"),
        ("../foo/bar/baz", "../foo/bar/baz"),
    ];

    for &(input, expected) in PATHS.iter() {
        assert_eq!(remove_leading_dot(input).to_str(), Some(expected));
    }
}


/// Tests wether a directory entry is a not-hidden file.
pub fn filter_file(entry: &DirEntry) -> bool {
    entry.file_name()
         .to_str()
         .and_then(|s| s.chars().next())
         .map(|c| !INVALID_CHARS.contains(c))
         .unwrap_or(false) && entry.file_type().is_file()
}

/// Tests wether a directory entry is an Handlebar template.
pub fn filter_template(entry: &DirEntry) -> bool {
    filter_file(entry) &&
    entry.file_name()
         .to_str()
         .map(|s| s.ends_with(TEMPLATE_EXTENSION))
         .unwrap_or(false)
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct DateTime(chrono::DateTime<FixedOffset>);

impl DateTime {
    pub fn from_string(s: &str) -> Option<DateTime> {
        chrono::DateTime::parse_from_rfc3339(s).ok().map(DateTime)
    }
}

impl Decodable for DateTime {
    fn decode<D: Decoder>(decoder: &mut D) -> result::Result<Self, D::Error> {
        let datetime_str: String = try! { String::decode(decoder) };
        chrono::DateTime::parse_from_rfc3339(datetime_str.as_ref())
            .map_err(|_| decoder.error("Malformed date"))
            .map(|d| DateTime(d))
    }
}

impl Encodable for DateTime {
    fn encode<S: Encoder>(&self, encoder: &mut S) -> Result<(), S::Error> {
        self.0.to_rfc3339().encode(encoder)
    }
}

impl ToJson for DateTime {
    fn to_json(&self) -> Json {
        Json::String(self.0.to_rfc3339())
    }
}



pub trait FromRaw where Self: 'static + Sized {
    fn from_raw(raw: &str) -> Option<Self>;
}

impl FromRaw for bool {
    fn from_raw(raw: &str) -> Option<bool> {
        match raw.trim().to_ascii_lowercase().as_ref() {
            "1" | "t" | "true" | "on" | "yes" | "j" | "jes" => Some(true),
            "0" | "f" | "false" | "off" | "non" | "n" | "ne" => Some(false),
            _ => None,
        }
    }
}

impl FromRaw for DateTime {
    fn from_raw(raw: &str) -> Option<DateTime> {
        DateTime::from_string(raw.trim())
    }
}


impl FromRaw for String {
    fn from_raw(raw: &str) -> Option<String> {
        Some(String::from(raw.trim()))
    }
}

impl FromRaw for Vec<String> {
    fn from_raw(raw: &str) -> Option<Vec<String>> {
        Some(raw.split(',').flat_map(|v| FromRaw::from_raw(v)).collect())
    }
}

impl<T> FromRaw for Option<T> where T: FromRaw {
    fn from_raw(raw: &str) -> Option<Option<T>> {
        T::from_raw(raw).map(Some)
    }
}
