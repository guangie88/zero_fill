use hound;
use std;
use std::fmt::Formatter;
use std::str::Utf8Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    CStrConv(Utf8Error),
    Filename,
    IO(std::io::Error),
    Wav(hound::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        "zero_fill::Error: {}"
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::IO(e)
    }
}

impl From<hound::Error> for Error {
    fn from(e: hound::Error) -> Error {
        Error::Wav(e)
    }
}