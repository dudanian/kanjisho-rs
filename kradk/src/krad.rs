use crate::{Error, NomError, Result};
use nom::{
    bytes::complete::tag, character::complete::anychar, combinator::rest, sequence::separated_pair,
};

/// A single KRAD Kanji/Radical entry
#[derive(Debug, PartialEq)]
pub struct Entry {
    /// The key kanji
    pub kanji: char,
    /// The list of radicals
    pub radicals: String,
}

/// Iterator over the entries of a KRAD file
pub fn iterator<'a>(input: &'a str) -> impl Iterator<Item = Result<Entry>> + 'a {
    input
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .filter(|l| !l.starts_with("#"))
        .map(|l| parse_line(l))
}

/// Parse a single KRAD entry
/// this currently doesn't guarantee the radical list is not empty
fn parse_line(i: &str) -> Result<Entry> {
    separated_pair(anychar, tag(" : "), rest)(i)
        .map(|(_, (kanji, radicals))| Entry {
            kanji,
            radicals: remove_spaces(radicals),
        })
        // owning the error is easier than dealing with the ref for now
        .or_else(|e: NomError<&str>| Err(Error::Parse(e.to_owned())))
}

/// Remove spaces from the KRAD radicals list
fn remove_spaces(i: &str) -> String {
    let mut s = i.to_owned();
    s.retain(|c| c != ' ');
    s
}
