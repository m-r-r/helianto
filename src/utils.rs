use std::{io, fs};
use std::path::Path;


#[doc(hidden)]
pub trait PathExt {
    fn metadata(&self) -> io::Result<fs::Metadata>;
    fn exists(&self) -> bool;
    fn is_file(&self) -> bool;
    fn is_dir(&self) -> bool;
    fn relative_from_<'a, P: ?Sized + AsRef<Path>>(&'a self, base: &'a P) -> Option<&Path>;
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

    fn relative_from_<'a, P: ?Sized + AsRef<Path>>(&'a self, base: &'a P) -> Option<&Path> {
        iter_after(self.components(), base.as_ref().components()).map(|c| c.as_path())
    }
}

#[doc(hidden)]
fn iter_after<A, I, J>(mut iter: I, mut prefix: J) -> Option<I>
    where I: Iterator<Item = A> + Clone,
          J: Iterator<Item = A>,
          A: PartialEq
{
    loop {
        let mut iter_next = iter.clone();
        match (iter_next.next(), prefix.next()) {
            (Some(x), Some(y)) => {
                if x != y {
                    return None;
                }
            }
            (Some(_), None) => return Some(iter),
            (None, None) => return Some(iter),
            (None, Some(_)) => return None,
        }
        iter = iter_next;
    }
}

static INVALID_CHARS: &'static str = "._~#$";


/// Check if a filename is valid
///
/// A valid filename must not start with `.`, `_`, `~`, `#` or `$` and must be at least one
/// character long.
pub fn valid_filename<S: AsRef<Path>>(filename: S) -> bool {
    filename.as_ref()
            .file_name()
            .and_then(|filename| filename.to_str())
            .and_then(|filename_str| filename_str.chars().next())
            .map(|first_char| !INVALID_CHARS.contains(first_char))
            .unwrap_or(false)
}
