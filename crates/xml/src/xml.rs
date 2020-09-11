//! XML definitions and aliases
//!
//! Define structs for XML elements as well as other useful types. Pretty much
//! everything here is used when parsing.
use std::fmt;
use std::fmt::Display;
use std::io;
use std::string::FromUtf8Error;

/// An XML starting tag with attributes
#[derive(Debug, PartialEq)]
pub struct StartTag {
    pub name: String,
    pub attrs: Vec<Attr>,
}

/// An XML name/value attribute pair
#[derive(Debug, PartialEq)]
pub struct Attr {
    pub name: String,
    pub value: String,
}

/// An XML processing instruction
#[derive(Debug, PartialEq)]
pub struct ProcInst {
    pub target: String,
    pub inst: String,
}

/// Public iterable XML tokens
#[derive(Debug, PartialEq)]
pub enum Token {
    StartTag(StartTag),
    EndTag(String),
    CharData(String),
    ProcInst(ProcInst),
    EndOfFile,
}

/// An XML result wrapper
pub type Result<T> = std::result::Result<T, Error>;

/// XML parsing error
#[derive(Debug)]
pub enum Error {
    MalformedAttlistDecl,
    MalformedAttValue,
    MalformedByteOrderMark,
    MalformedCData,
    MalformedCharData,
    MalformedCharRef,
    MalformedComment,
    MalformedDoctype,
    MalformedDoctypeEntity,
    MalformedEmptyElemTag,
    MalformedEndTag,
    MalformedEntityDecl,
    MalformedEntityRef,
    MalformedEntityValue,
    MalformedEq,
    MalformedExternalEntity,
    MalformedName,
    MalformedProcInst,
    MalformedStartTag,
    MalformedSystemLiteral,
    MalformedVersionLiteral,
    MalformedXmlDecl,
    MalformedYesNoLiteral,
    MismatchingStartEndTags,
    IoError(io::Error),
    Utf8Error(FromUtf8Error),
    UnexpectedEof,
    UnexpectedToken,
    UnmappedEntityRef,
    UnsupportedEncoding,
    UnsupportedFeature(Feature),
}

#[derive(Debug)]
pub enum Feature {
    ExternalEntities,
    ParameterEntities,
    Notations,
}

// TODO descriptive error strings
impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            _ => formatter.write_str("some error"),
        }
    }
}

impl std::error::Error for Error {}

impl From<FromUtf8Error> for Error {
    fn from(error: FromUtf8Error) -> Self {
        Error::Utf8Error(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IoError(error)
    }
}
