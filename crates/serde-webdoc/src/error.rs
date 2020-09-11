use serde::de;
use std::{error, fmt};

/// `Result` wrapper for `Error`
pub type Result<T> = std::result::Result<T, Error>;

/// Parsing `Error`s
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    Message(String),
    NoElement,
    NoTextContent,
    NonEmptyUnit,
    ParseError,
}

impl de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;

        match self {
            Message(s) => formatter.write_str(&s),
            NoElement => formatter.write_str("no element to read from"),
            NoTextContent => formatter.write_str("element contains no text data"),
            NonEmptyUnit => formatter.write_str("expected element to have no contents"),
            ParseError => formatter.write_str("error while parsing type"),
        }
    }
}

impl error::Error for Error {}
