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

use chrono::{self, FixedOffset};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::path::{Component, Path, PathBuf};

fn is_hidden<S: AsRef<Path> + Sized>(path: &S) -> bool {
    path.as_ref()
        .file_name()
        .and_then(|osstr| osstr.to_str())
        .map(|file_name| file_name.starts_with('.'))
        .unwrap_or(false)
}

pub fn is_public<S: AsRef<Path> + Sized>(path: &S) -> bool {
    !is_hidden(path)
        && path
            .as_ref()
            .file_name()
            .and_then(|osstr| osstr.to_str())
            .map(|file_name| !file_name.starts_with('_'))
            .unwrap_or(false)
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
        path_ref
            .components()
            .skip(1)
            .map(Component::as_os_str)
            .collect()
    } else {
        path_ref.into()
    }
}

#[test]
fn test_remove_leading_dot() {
    const PATHS: &[(&str, &str)] = &[
        ("/foo/bar/baz", "/foo/bar/baz"),
        ("foo/bar/baz", "foo/bar/baz"),
        ("./foo/bar/baz", "foo/bar/baz"),
        ("../foo/bar/baz", "../foo/bar/baz"),
    ];

    for &(input, expected) in PATHS.iter() {
        assert_eq!(remove_leading_dot(input).to_str(), Some(expected));
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DateTime(
    #[serde(
        serialize_with = "serialize_date",
        deserialize_with = "deserialize_date"
    )]
    chrono::DateTime<FixedOffset>,
);

fn deserialize_date<'de, D>(deserializer: D) -> Result<chrono::DateTime<FixedOffset>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    chrono::DateTime::parse_from_rfc3339(s.as_str()).map_err(serde::de::Error::custom)
}

fn serialize_date<S>(date: &chrono::DateTime<FixedOffset>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(date.to_rfc3339().as_str())
}

impl DateTime {
    pub fn from_string(s: &str) -> Option<DateTime> {
        chrono::DateTime::parse_from_rfc3339(s).ok().map(DateTime)
    }
}



pub trait FromRaw where Self: 'static + Sized ,
{
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

impl<T> FromRaw for Option<T>
where
    T: FromRaw,
{
    fn from_raw(raw: &str) -> Option<Option<T>> {
        T::from_raw(raw).map(Some)
    }
}
