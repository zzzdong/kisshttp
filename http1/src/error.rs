use std::fmt;

use crate::parser;

#[derive(Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Io,
    Protocol,
}

pub struct Error {
    kind: ErrorKind,
    cause: Box<dyn std::error::Error + Send + 'static>,
}

impl Error {
    pub fn new(kind: ErrorKind, cause: impl std::error::Error + Send + 'static) -> Self {
        Error {
            kind,
            cause: Box::new(cause),
        }
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Error")
            .field("kind", &self.kind)
            .field("cause", &self.cause)
            .finish()
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Error")
            .field("kind", &self.kind)
            .field("cause", &self.cause)
            .finish()
    }
}



impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::new(ErrorKind::Io, err)
    }
}

impl From<parser::ParseError> for Error {
    fn from(err: parser::ParseError) -> Self {
        Error::new(ErrorKind::Protocol, err)
    }
}

