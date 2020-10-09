use std::io;

#[derive(Debug)]
pub enum ParsingError {
    IoError(io::Error),
}

impl From<io::Error> for ParsingError {
    fn from(io_err: io::Error) -> Self {
        ParsingError::IoError(io_err)
    }
}

impl From<ParsingError> for String {
    fn from(err: ParsingError) -> Self {
        format!("{:?}", err).to_owned()
    }
}
