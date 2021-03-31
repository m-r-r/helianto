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

use super::Settings;
use serde::{Deserialize, Serialize};
use std::default::Default;

#[derive(Debug, Serialize, Deserialize)]
pub struct Site {
    pub title: String,
    pub language: Option<String>,
    pub url: String,
}

impl Default for Site {
    fn default() -> Site {
        Site {
            title: "Untitled website".into(),
            language: None,
            url: "/".into(),
        }
    }
}

impl Site {
    pub fn new(settings: &Settings) -> Site {
        Site {
            title: settings.site_title.clone(),
            url: settings.site_url.clone(),
            language: settings.site_language.clone(),
        }
    }
}
