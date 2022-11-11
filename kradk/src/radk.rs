use crate::{Error, NomError, Result};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{u8, anychar, char, multispace1, newline, not_line_ending},
    combinator::{rest, value},
    sequence::{separated_pair, tuple},
};

/// A single RADK Radical/Kanji entry
#[derive(Debug, PartialEq)]
pub struct Entry {
    /// The key kanji
    pub radical: char,
    /// The list of radicals
    pub kanjis: String,
}

struct MyIter<'a, I: Iterator<Item = &'a str>> {
    // need the line iterator
    // wtf is this going to look like?
    iter: I,
}

impl<'a, I: Iterator<Item = &'a str>> Iterator for MyIter<'a, I> {
    type Item = Entry;

    fn next(&mut self) -> Option<Self::Item> {
        let n = self.iter.next()?;

        // read first line as char/int
        // read all lines until one starts with $
        

        None
    }
}

/// Iterator over the entries of a RADK file
pub fn iterator<'a>(input: &'a str) -> impl Iterator<Item = Result<Entry>> + 'a {
    let it = input
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .filter(|l| !l.starts_with("#"));
    //.map(|l| parse_line(l));
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

#[derive(Debug, PartialEq, Clone)]
enum Token<'a> {
    Comment,
    Entry(char, i32),
    Line(&'a str)
}

fn parse_root(i: &str) -> nom::IResult<&str, ()> {
    alt((parse_comment, entry))(i)
}

fn parse_comment(i: &str) -> nom::IResult<&str, Token> {
    value(
        Token::Comment,
        tag("#"), // TODO fix this
    )(i)
}

fn parse_entry(i: &str) -> nom::IResult<&str, Token> {
    tuple((char('$'), multispace1, u8, rest))(i)
    // XXX how do you map again?
}

/// Parser for single header line
/// like '$ 化 2 js01'
/// that extra bit probably means I should
/// manually replace with a better unicode char

/// Parser for character block

/// Remove spaces from the KRAD radicals list
fn remove_spaces(i: &str) -> String {
    let mut s = i.to_owned();
    s.retain(|c| c != ' ');
    s
}
