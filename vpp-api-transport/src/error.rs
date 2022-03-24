use std::error;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    Error(String),
    InvalidHeader,
    InvalidMessage,
    IoError(std::io::Error),
    BinCodeError(Box<bincode::ErrorKind>),
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error: {:?}", self)
    }
}
impl From<Box<bincode::ErrorKind>> for Error {
    fn from(e: Box<bincode::ErrorKind>) -> Self {
        Self::BinCodeError(e)
    }
}
impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Self::Error(s.to_string())
    }
}
impl From<String> for Error {
    fn from(s: String) -> Self {
        Self::Error(s)
    }
}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
