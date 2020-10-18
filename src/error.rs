use std::io;
use std::num;
use std::str;

pub type Result<T> = std::result::Result<T, ParsingError>;

#[derive(Debug)]
pub enum ParsingError {
    ContentError(String),
    IoError(io::Error),
    ParseIntError(num::ParseIntError),
    Utf8Error(str::Utf8Error),
    BincodeError(Box<bincode::ErrorKind>),
    IpadicBuilderError(lindera_ipadic_builder::ParsingError),
}

impl From<String> for ParsingError {
    fn from(err: String) -> Self {
        ParsingError::ContentError(err)
    }
}

impl From<io::Error> for ParsingError {
    fn from(err: io::Error) -> Self {
        ParsingError::IoError(err)
    }
}

impl From<num::ParseIntError> for ParsingError {
    fn from(err: num::ParseIntError) -> Self {
        ParsingError::ParseIntError(err)
    }
}

impl From<str::Utf8Error> for ParsingError {
    fn from(err: str::Utf8Error) -> Self {
        ParsingError::Utf8Error(err)
    }
}

impl From<Box<bincode::ErrorKind>> for ParsingError {
    fn from(err: Box<bincode::ErrorKind>) -> Self {
        ParsingError::BincodeError(err)
    }
}

impl From<lindera_ipadic_builder::ParsingError> for ParsingError {
    fn from(err: lindera_ipadic_builder::ParsingError) -> Self {
        ParsingError::IpadicBuilderError(err)
    }
}

impl From<ParsingError> for String {
    fn from(err: ParsingError) -> Self {
        format!("{:?}", err)
    }
}
