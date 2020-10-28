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


use std::{error, fmt, result};
use std::io::Error as IoError;
use std::path::{Path, PathBuf};
use std::borrow::Borrow;
use toml;


pub type Result<T> = result::Result<T, Error>;


#[derive(Debug)]
pub enum Error {
    Io(IoError),
    // An error happened while reading a document.
    Reader {
        path: PathBuf,
        cause: Box<error::Error>,
    },

    // An error happened while rendering a file
    Render {
        cause: Box<error::Error>,
    },

    // An error happened while copying a file
    Copy {
        from: PathBuf,
        to: PathBuf,
        cause: Box<error::Error>,
    },

    // An error happened while writing an output file
    Output {
        dest: PathBuf,
        cause: Box<error::Error>,
    },

    // An error happened while reading the configuration file
    LoadSettings {
        path: PathBuf,
        cause: Box<error::Error>,
    },

    // The software is misconfigured
    Settings {
        message: String,
    },

    // An error happened while trying to parse a date supplied by the user
    InvalidDate {
        date: String,
    },

    // A document contains an unkown metadata field
    UnknownMetadataField {
        name: String,
    },
}


impl From<IoError> for Error {
    fn from(error: IoError) -> Error {
        Error::Io(error)
    }
}


impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref err) => write!(f, "IO error: {}", err),
            Error::Reader { ref path, ref cause} =>
                write!(f, "Error while reading {}: {}", path.display(), cause),
            Error::Copy { ref from, ref to, ref cause } => write!(f,
                                                                  "Could not copy {} to {}: {}",
                                                                  from.display(),
                                                                  to.display(),
                                                                  cause),
            Error::Output { ref dest, ref cause } => write!(f,
                                                            "Could not write output file {}: {}",
                                                            dest.display(),
                                                            cause),
            Error::Render { ref cause } => write!(f, "Rendering failed: {}", cause),
            Error::LoadSettings { ref path, ref cause } => write!(f,
                       "Could not read settings file {}: {}",
                       path.display(),
                       cause),
            Error::InvalidDate { ref date } =>
                write!(f, "\"{}\" is not a valid date.", date.trim()),
            Error::UnknownMetadataField { ref name } => write!(f, "Unknown metadata \"{}\".", name),
            Error::Settings { ref message } => write!(f, "{}", message),
        }
    }
}


impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref err) => err.description(),
            Error::Reader { ref cause, .. } => cause.description(),
            Error::Copy { ref cause, .. } => cause.description(),
            Error::Output { ref cause, .. } => cause.description(),
            Error::Render { ref cause, .. } => cause.description(),
            Error::LoadSettings { ref cause, .. } => cause.description(),
            Error::InvalidDate { .. } => "Invalid date",
            Error::UnknownMetadataField { .. } => "Unknown metadata field",
            Error::Settings { .. } => "Invalid configuration",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Io(ref err) => Some(err),
            Error::Reader { ref cause, .. } => Some(cause.borrow()),
            Error::Copy { ref cause, .. } => Some(cause.borrow()),
            Error::Output { ref cause, .. } => Some(cause.borrow()),
            Error::Render { ref cause, .. } => Some(cause.borrow()),
            Error::LoadSettings { ref cause, .. } => Some(cause.borrow()),
            _ => None,
        }
    }
}
