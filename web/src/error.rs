use std::fmt;
use std::fmt::Display;

pub enum Error {
    LocalStorageWrite,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::LocalStorageWrite => "Local Storage Write Failed",
            }
        )
    }
}
