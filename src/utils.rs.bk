use std::{io, fs};
use std::path::Path;
use std::os::OsStr;

#[doc(hidden)]
pub trait PathExt {
    fn metadata(&self) -> io::Result<fs::Metadata>;
    fn exists(&self) -> bool;
    fn is_file(&self) -> bool;
    fn is_dir(&self) -> bool;
}

#[doc(hidden)]
impl PathExt for Path {
    fn metadata(&self) -> io::Result<fs::Metadata> {
        fs::metadata(self)
    }
    fn exists(&self) -> bool {
        fs::metadata(self).is_ok()
    }

    fn is_file(&self) -> bool {
        fs::metadata(self).map(|s| s.is_file()).unwrap_or(false)
    }

    fn is_dir(&self) -> bool {
        fs::metadata(self).map(|s| s.is_dir()).unwrap_or(false)
    }
}

static INVALID_CHARS: &'static str = "._~#$";


/// Check if a filename is valid
///
/// A valid filename must not start with `.`, `_`, `~`, `#` or `$` and must be at least one
/// character long.
pub fn valid_filename<S: AsRef<OsStr>>(filename: S) -> bool {
    filename.to_str()
            .map(|filename_str| {
                if filename_str.len() > 0 {
                    !INVALID_CHARS.contains(filename_str.char_at(0))
                } else {
                    false
                }
            })
            .unwrap_or(false)
}
