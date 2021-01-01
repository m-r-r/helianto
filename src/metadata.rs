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

use crate::{Error, Result};

use serde::{Serialize, Deserialize};
use chrono::{self, FixedOffset};
use serde_yaml::{self, from_str};
use regex::Regex;

#[derive(Clone, Eq, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Value(serde_yaml::Value);

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DateTime(chrono::DateTime<FixedOffset>);


lazy_static! {
    static ref RE: Regex = Regex::new(r#"(?sm:\A---\r?\n(.*)^---\r?\n)"#).unwrap();
}
pub fn parse_frontmatter<T>(input: &str) -> Result<(Option<T>, &str)> {
    match RE.captures(input).and_then(|s| s.get(1)) {
        Some(m) => Ok((from_str::<Option<T>>(m.as_str())?, input[m.end()..].trim_start())),
        None => Ok((None, input)),
    }
}
