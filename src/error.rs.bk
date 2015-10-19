use std::{error, result, fmt};
use std::io::Error as IoError;
use std::path::PathBuf;


pub type Result<T> = result::Result<T, Error>;


#[derive(Debug)]
pub enum Error {
    Io(IoError),
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
        }
    }
}


impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Io(ref err) => Some(err),
        }
    }
}
