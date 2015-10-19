use std::{error, result, fmt};
use std::io::Error as IoError;
use std::path::PathBuf;
use std::borrow::Borrow;


pub type Result<T> = result::Result<T, Error>;


#[derive(Debug)]
pub enum Error {
    Io(IoError),
    // An error happened while reading a document.
    Reader {
        path: PathBuf,
        cause: Box<error::Error>,
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
            Error::Reader{ref path, ref cause} =>
                write!(f, "Error while reading {}: {}", path.display(), cause),
        }
    }
}


impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref err) => err.description(),
            Error::Reader { ref cause, .. } => cause.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Io(ref err) => Some(err),
            Error::Reader { ref cause, .. } => Some(cause.borrow()),
        }
    }
}
