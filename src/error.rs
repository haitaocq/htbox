use std::fmt;

#[derive(Debug)]
pub struct Error;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error")
    }
}

impl std::error::Error for Error {}
