//! Parser for XML. Somewhat based off of the XML parser from Golang.
//!
//! # Examples
//! ```
//! # use xml::read::Parser;
//! let text = "<test attr='value'>hello</test>";
//! let mut reader = Parser::from_reader(text.as_bytes());
//! for tok in reader {
//!     println!("{:?}", tok);
//! }
//! ```
#[macro_use]
mod macros;
mod bytes;
#[cfg(not(feature = "disable-dtd-parsing"))]
mod doctype;

use crate::xml::{self, Error, Feature, Token};

use bytes::Bytes;
use std::char;
use std::io::Read;

#[cfg(feature = "custom-entities")]
use std::collections::HashMap;

/// Enum to select how we should read text blocks.
#[derive(PartialEq)]
enum TextMode {
    CharData,
    AttValue,
    EntityValue,
    //Normal,
    //Double,
    //Single,
}

/// Enum to limit the radix values used by char refs.
enum CharMode {
    Decimal = 10,
    Hex = 16,
}

/// The states of the xml decoder.
/// Each state expects a different set of raw tokens.
enum State {
    // The very beginning of the document
    Begin,
    // The xml prologue
    Prologue,
    // Inside an internal DTD
    InternalDoctype,
    // Inside the root element
    Normal,
    // After the root element
    Epilogue,
    // After valid EOF
    End,
}

enum ExternalEntity {
    System(String),
    Public(String, String),
}

/// Internal raw tokens.
/// Used by our state machine for parsing.
#[derive(Debug, PartialEq)]
enum RawToken {
    XmlDecl,
    Comment,
    ProcInst(xml::ProcInst),
    DoctypeDef(Doctype),
    StartTag(xml::StartTag),
    EndTag(String),
    CharData(String),
}

/// Doctype token variants.
#[derive(Debug, PartialEq)]
pub enum Doctype {
    /// A doctype decl
    /// bool is whether or not there is an internal subset
    DoctypeDecl(bool),
    EntityDecl,
    ElementDecl,
    AttlistDecl,
    NotationDecl,
    /// The end of the internal subset
    InternalEnd,
}

/// Xml Decoder / Parser
pub struct Parser<R: Read> {
    /// XML byte reader
    bytes: Bytes<R>,
    /// Internal state
    state: State,
    /// Last token
    last_token: Option<RawToken>,
    /// Open tag stack
    tags: Vec<String>,
    /// Auto-close tag
    auto_close: Option<String>,
    /// Map of custom entities
    #[cfg(feature = "custom-entities")]
    custom_entities: HashMap<String, String>,
}

/// Token parsing functions
impl<R: Read> Parser<R> {
    /// Create a new XML parser from a reader stream
    pub fn from_reader(reader: R) -> Parser<R> {
        Parser {
            bytes: Bytes::from_reader(reader),
            state: State::Begin,
            last_token: None,
            tags: Vec::new(),
            auto_close: None,
            #[cfg(feature = "custom-entities")]
            custom_entities: HashMap::new(),
        }
    }

    /// Get the next public token from the source.
    ///
    /// Will continue to pass `Token::EndOfFile` once the stream is over. Will
    /// be in an invalid state after an error, do not try to read tokens after
    /// an error.
    pub fn token(&mut self) -> xml::Result<Token> {
        use Doctype::*;
        use RawToken::*;
        use State::*;

        loop {
            match self.state {
                // State::Begin
                // we check for the byte order mark and look out for a possible
                // xml-decl. note that we technically can't have an xml-decl if
                // we read any spacing before a token since it should be the
                // absolute first element in the document
                Begin => {
                    self.byte_order_mark()?;

                    if !self.whitespace()? {
                        match self.raw_token()? {
                            XmlDecl => (),
                            t => self.last_token = Some(t),
                        }
                    }
                    self.state = Prologue;
                }

                // State::Prologue
                // here we read the rest of the prolog from after the optional
                // xml-decl, looking out for an internal DTD

                // we need to check the last token here because we do token
                // lookahead in the state before.
                Prologue => {
                    let token = if self.last_token.is_some() {
                        self.last_token.take().unwrap()
                    } else {
                        // ignoring spaces first lets us fail on CharData,
                        // otherwise we would have to verify that all char data
                        // consists of only spaces
                        self.whitespace()?;
                        self.raw_token()?
                    };

                    match token {
                        DoctypeDef(DoctypeDecl(true)) => self.state = InternalDoctype,
                        DoctypeDef(DoctypeDecl(false)) => (),
                        StartTag(s) => {
                            self.tags.push(s.name.to_owned());
                            self.state = Normal;
                            return Ok(Token::StartTag(s));
                        }
                        Comment => (),
                        ProcInst(p) => return Ok(Token::ProcInst(p)),
                        _ => return Err(Error::UnexpectedToken),
                    }
                }
                // State::InternalDoctype
                // here we parse all of the doctype tokens. since internal DTD
                // syntax is so specialized, we use a different token parser.
                // all actual DTD processing is done internally so each doctype
                // token is more of an indication that work has been done
                // internally
                InternalDoctype => {
                    self.whitespace()?;
                    match self.doctype_token()? {
                        DoctypeDef(DoctypeDecl(_)) => return Err(Error::UnexpectedToken),
                        DoctypeDef(InternalEnd) => self.state = Prologue,
                        DoctypeDef(_) => (),
                        Comment => (),
                        ProcInst(p) => return Ok(Token::ProcInst(p)),
                        _ => return Err(Error::UnexpectedToken),
                    }
                }
                // State::Normal
                // here we are inside the root element and are parsing the
                // actual xml data. whitespace between elements is now treated
                // as char data and passed up to the application
                Normal => match self.raw_token()? {
                    StartTag(s) => {
                        self.tags.push(s.name.to_owned());
                        return Ok(Token::StartTag(s));
                    }
                    EndTag(name) => {
                        let start_name = self.tags.pop().unwrap();
                        if start_name != name {
                            return Err(Error::MismatchingStartEndTags);
                        }

                        if self.tags.is_empty() {
                            self.state = Epilogue;
                        }
                        return Ok(Token::EndTag(name));
                    }
                    CharData(d) => return Ok(Token::CharData(d)),
                    Comment => (),
                    ProcInst(p) => return Ok(Token::ProcInst(p)),
                    _ => return Err(Error::UnexpectedToken),
                },
                // State::Epilogue
                // here we just parse whatever few elements are still allowed
                // after the root element and look out for EndOfFile errors
                // between elements
                Epilogue => {
                    match self.whitespace() {
                        Ok(_) => (),
                        Err(Error::UnexpectedEof) => {
                            self.state = End;
                            return Ok(Token::EndOfFile);
                        }
                        Err(e) => return Err(e),
                    }
                    match self.raw_token()? {
                        Comment => (),
                        ProcInst(p) => return Ok(Token::ProcInst(p)),
                        _ => return Err(Error::UnexpectedToken),
                    }
                }
                // State::End
                // here we parsed everything with no errors, just keep returing
                // EndOfFile
                End => return Ok(Token::EndOfFile),
            }
        }
    }

    /// Parse a raw token from the input stream.
    ///
    /// Reads the minimum number of bytes to delegate to a specialized token
    /// parsing function.
    fn raw_token(&mut self) -> xml::Result<RawToken> {
        if let Some(name) = self.auto_close.take() {
            return Ok(RawToken::EndTag(name));
        }

        match self.next_byte()? {
            b'<' => match self.next_byte()? {
                b'/' => self.end_tag(),
                b'?' => self.proc_inst(),
                b'!' => match self.next_byte()? {
                    b'-' => self.comment(),
                    b'[' => self.cdata(),
                    b => {
                        self.unget_byte(b);
                        self.doctype()
                    }
                },
                b => {
                    self.unget_byte(b);
                    self.start_tag()
                }
            },
            b => {
                self.unget_byte(b);
                self.char_data()
            }
        }
    }

    /// Parse a comment. Does not return the actual comment text.
    ///
    /// # Assumptions
    /// Assumes that `<!-` has already been read.
    ///
    /// # Grammar
    /// ```text
    /// Comment  ::=  '<!--' ((Char - '-') | ('-' (Char - '-')))* '-->'
    /// ```
    fn comment(&mut self) -> xml::Result<RawToken> {
        require_byte!(b'-', self, Error::MalformedComment);
        loop {
            if self.next_byte_is(b'-')? && self.next_byte_is(b'-')? {
                require_byte!(b'>', self, Error::MalformedComment);
                return Ok(RawToken::Comment);
            }
        }
    }

    /// Parse a processing instruction (including xml-decl).
    ///
    /// # Assumptions
    /// Assumes that `<?` has already been read.
    ///
    /// # Grammar
    /// ```text
    /// PI        ::=  '<?' PITarget (S (Char* - (Char* '?>' Char*)))? '?>'
    /// PITarget  ::=  Name - (('X' | 'x') ('M' | 'm') ('L' | 'l'))
    /// ```
    fn proc_inst(&mut self) -> xml::Result<RawToken> {
        let target = self.name()?;

        if Self::has_xml_prefix(&target) {
            return match target.as_ref() {
                "xml" => self.xml_decl(),
                _ => Err(Error::MalformedProcInst),
            };
        }

        let mut buf = Vec::new();

        if self.whitespace()? {
            while !buf.ends_with(b"?>") {
                buf.push(self.next_byte()?);
            }
            buf.truncate(buf.len() - 2);
        }

        Ok(RawToken::ProcInst(xml::ProcInst {
            target,
            inst: String::from_utf8(buf)?,
        }))
    }

    /// Parse an xml declaration.
    ///
    /// # Assumptions
    /// Assumes that `<?xml` has already been read.
    ///
    /// # Grammar
    /// ```text
    /// XMLDecl       ::=  '<?xml' VersionInfo EncodingDecl? SDDecl? S? '?>'
    /// VersionInfo   ::=  S 'version' Eq VersLiteral
    /// EncodingDecl  ::=  S 'encoding' Eq EncLiteral
    /// SDDecl        ::=  S 'standalone' Eq YesNoLiteral
    /// ```
    fn xml_decl(&mut self) -> xml::Result<RawToken> {
        require_whitespace!(self, Error::MalformedXmlDecl);
        require_str!("version", self, Error::MalformedXmlDecl);
        self.equals()?;
        let _minor = self.version_literal()?;

        let mut encoding = None;
        let mut standalone = None;

        loop {
            let sp = self.whitespace()?;
            match self.next_byte()? {
                b'e' if sp && encoding.is_none() && standalone.is_none() => {
                    require_str!("ncoding", self, Error::MalformedXmlDecl);
                    self.equals()?;
                    encoding = Some(self.encoding_literal()?);
                }
                b's' if sp && standalone.is_none() => {
                    require_str!("tandalone", self, Error::MalformedXmlDecl);
                    self.equals()?;
                    standalone = Some(self.yesno_literal()?);
                }
                b'?' => {
                    require_byte!(b'>', self, Error::MalformedXmlDecl);
                    break;
                }
                _ => return Err(Error::MalformedXmlDecl),
            }
        }

        if let Some(mut encoding) = encoding {
            encoding.make_ascii_lowercase();
            if encoding != "utf-8" {
                return Err(Error::UnsupportedEncoding);
            }
        }

        // TODO check standalone and version

        Ok(RawToken::XmlDecl)
    }

    /// Parse a doctype declaration.
    ///
    /// # Usage
    /// This function will stop parsing before the internal subset if one
    /// exists, which means there is a chance that after this function is
    /// called, the byte parser is not on an element boundary.
    ///
    /// # Assumptions
    /// Assumes that `<!` has already been read.
    ///
    /// # Grammar
    /// ```text
    /// doctypedecl  ::=  '<!DOCTYPE' S Name (S ExternalID)? S? ('[' intSubset ']' S?)? '>'
    /// ```
    fn doctype(&mut self) -> xml::Result<RawToken> {
        require_str!("DOCTYPE", self, Error::MalformedDoctype);
        require_whitespace!(self, Error::MalformedDoctype);
        let name = self.name()?;

        if self.whitespace()? {
            if let Some(_) = self.external_entity()? {
                // TODO we currently don't support external entities
                return Err(Error::UnsupportedFeature(Feature::ExternalEntities));
            }
        }

        match self.next_byte()? {
            b'[' => Ok(RawToken::DoctypeDef(Doctype::DoctypeDecl(true))),
            b'>' => Ok(RawToken::DoctypeDef(Doctype::DoctypeDecl(false))),
            _ => Err(Error::MalformedDoctype),
        }
    }

    /// Parse a doctype token from an internal subset.
    ///
    /// # Usage
    /// This function ignores the internal subset. A proper token parser will be
    /// provided separately. We can pretty reliably ignore the whole set as long
    /// as we respect quoted strings and look for the internal subset ending
    /// marker ']' S? '>'
    #[cfg(feature = "disable-dtd-parsing")]
    fn doctype_token(&mut self) -> xml::Result<RawToken> {
        loop {
            match self.next_byte()? {
                b @ b'\'' | b @ b'"' => self.read_until(b)?,
                b']' => {
                    self.whitespace()?;
                    require_byte!(b'>', self, Error::MalformedDoctype);
                    break;
                }
                _ => (),
            }
        }

        Ok(RawToken::DoctypeDef(Doctype::InternalEnd))
    }

    /// Parse a Start-Tag or Empty-Elem-Tag element.
    ///
    /// # Assumptions
    /// Assumes that `<` has already been read.
    ///
    /// # Grammar
    /// ```text
    /// EmptyElemTag  ::=  '<' Name (S Attribute)* S? '/>'
    /// STag          ::=  '<' Name (S Attribute)* S? '>'
    /// ```
    fn start_tag(&mut self) -> xml::Result<RawToken> {
        let name = self.name()?;
        let mut attrs = Vec::new();

        loop {
            let sp = self.whitespace()?;

            match self.next_byte()? {
                b'/' => {
                    require_byte!(b'>', self, Error::MalformedEmptyElemTag);
                    self.auto_close = Some(name.to_owned());
                    break;
                }
                b'>' => break,
                b if sp => {
                    self.unget_byte(b);
                    attrs.push(self.attr()?);
                }
                _ => return Err(Error::MalformedStartTag),
            }
        }

        Ok(RawToken::StartTag(xml::StartTag { name, attrs }))
    }

    /// Parse an End-Tag element.
    ///
    /// # Assumptions
    /// Assumes that `</` has already been read.
    ///
    /// # Grammar
    /// ```text
    /// ETag  ::=  '</' Name S? '>'
    /// ```
    fn end_tag(&mut self) -> xml::Result<RawToken> {
        let name = self.name()?;
        self.whitespace()?;
        require_byte!(b'>', self, Error::MalformedEndTag);

        Ok(RawToken::EndTag(name))
    }

    /// Parse a char data segment. This is actually a subset of xml content
    /// which expands references in contiguous char data segments.
    ///
    /// # TODO
    /// Should probably join these and cdata blocks together.
    ///
    /// # Grammar
    /// ```text
    /// content   ::=  CharData? ((Reference) CharData?)*
    /// CharData  ::=  [^<&]* - ([^<&]* ']]>' [^<&]*)
    /// ```
    fn char_data(&mut self) -> xml::Result<RawToken> {
        Ok(RawToken::CharData(self.expanded_text(TextMode::CharData)?))
    }

    /// Parse a cdata element.
    ///
    /// # Assumptions
    /// Assumes '<![' has already been read.
    ///
    /// # Grammar
    /// ```text
    /// CDSect   ::=  CDStart CData CDEnd
    /// CDStart  ::=  '<![CDATA['
    /// CData    ::=  (Char* - (Char* ']]>' Char*))
    /// CDEnd    ::=  ']]>'
    /// ```
    fn cdata(&mut self) -> xml::Result<RawToken> {
        require_str!("CDATA[", self, Error::MalformedCData);
        let mut buf = Vec::new();

        while !buf.ends_with(b"]]>") {
            buf.push(self.next_byte()?);
        }

        buf.truncate(buf.len() - 3);
        Ok(RawToken::CharData(String::from_utf8(buf)?))
    }
}

// Parsing helpers
impl<R: Read> Parser<R> {
    /// Read and verify the byte order mark is utf-8 if it exists.
    ///
    /// # Markers
    /// Bytes         | Encoding Form
    /// --------------|--------------
    /// `EF BB BF`    | UTF-8
    /// `FE FF`       | UTF-16, big-endian
    /// `FF FE`       | UTF-16, little-endian
    /// `00 00 FE FF` | UTF-32, big-endian
    /// `FF FE 00 00` | UTF-32, little-endian
    fn byte_order_mark(&mut self) -> xml::Result<()> {
        match self.next_byte()? {
            0xEF => {
                // assume utf-8 BOM
                if self.next_byte_is(0xBB)? && self.next_byte_is(0xBF)? {
                    Ok(())
                } else {
                    Err(Error::MalformedByteOrderMark)
                }
            }
            // starters for UTF-16 and UTF-32 BOMs
            0x00 | 0xFE | 0xFF => Err(Error::UnsupportedEncoding),
            b => {
                self.unget_byte(b);
                Ok(())
            }
        }
    }

    /// Returns whether the target begins with any casing of `xml`, since those
    /// target names are reserved by the spec.
    fn has_xml_prefix(target: &str) -> bool {
        if target.len() >= 3 {
            let mut it = target.bytes();
            for b in "xml".bytes() {
                if it.next().unwrap().to_ascii_lowercase() != b {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }

    /// Skips all whitespace characters and returns true if we skip any
    /// characters. Helps to distinguish between optional and mandatory
    /// whitespace.
    ///
    /// # Grammar
    /// ```text
    /// S  ::=  (#x20 | #x9 | #xD | #xA)+
    /// ```
    fn whitespace(&mut self) -> xml::Result<bool> {
        let mut skipped = false;
        loop {
            match self.next_byte()? {
                b' ' | b'\t' | b'\r' | b'\n' => {
                    skipped = true;
                }
                b => {
                    self.unget_byte(b);
                    return Ok(skipped);
                }
            }
        }
    }

    /// Skips whitespace before and after a mandatory equals sign.
    ///
    /// # Grammar
    /// ```text
    /// Eq  ::=  S? '=' S?
    /// ```
    fn equals(&mut self) -> xml::Result<()> {
        self.whitespace()?;
        require_byte!(b'=', self, Error::MalformedEq);
        self.whitespace()?;
        Ok(())
    }

    /// Check the next bytes against the provided string, returning whether they
    /// matched or not.
    ///
    /// # Usage
    /// This function is not meant to be used by itself since it leaves the byte
    /// reader in an invalid state on `false`. Use the `require_str` macro
    /// instead.
    unsafe fn expect_str(&mut self, s: &'static str) -> xml::Result<bool> {
        for b in s.bytes() {
            if !self.next_byte_is(b)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Parse a name and return it as a string.
    ///
    /// # TODO
    /// This function does not currently respect the below grammar. It greedily
    /// creates a name using known single byte name chars and will break on
    /// invalid single byte chars like `<` and ` `, but will happily accept any
    /// multi byte sequences. But at the very least utf-8 validity is checked.
    ///
    /// # Grammar
    /// ```text
    /// NameStartChar  ::=  ":" | [A-Z] | "_" | [a-z]
    ///                   | [#xC0-#xD6] | [#xD8-#xF6]
    ///                   | [#xF8-#x2FF] | [#x370-#x37D]
    ///                   | [#x37F-#x1FFF] | [#x200C-#x200D]
    ///                   | [#x2070-#x218F] | [#x2C00-#x2FEF]
    ///                   | [#x3001-#xD7FF] | [#xF900-#xFDCF]
    ///                   | [#xFDF0-#xFFFD] | [#x10000-#xEFFFF]
    /// NameChar       ::=  NameStartChar | "-" | "." | [0-9] | #xB7
    ///                   | [#x0300-#x036F] | [#x203F-#x2040]
    /// Name           ::=  NameStartChar (NameChar)*
    /// ```
    fn name(&mut self) -> xml::Result<String> {
        let mut buf = Vec::new();

        loop {
            let b = match self.next_byte()? {
                b @ b'A'..=b'Z'
                | b @ b'a'..=b'z'
                | b @ b'0'..=b'9'
                | b @ b'_'
                | b @ b':'
                | b @ b'.'
                | b @ b'-' => b,
                b if b <= 0x80 => {
                    self.unget_byte(b);
                    break;
                }
                b => b,
            };

            buf.push(b);
        }

        if buf.is_empty() {
            return Err(Error::MalformedName);
        }

        // TODO this guarantees that the data is valid utf-8 but does not
        // guarantee that chars are in the allowed xml range or that start chars
        // are honored
        Ok(String::from_utf8(buf)?)
    }

    /// Parse an attribute, returning it as an `xml::Attr`
    ///
    /// # Grammar
    /// ```text
    /// Attribute  ::=  Name Eq AttValue
    /// ```
    fn attr(&mut self) -> xml::Result<xml::Attr> {
        let name = self.name()?;
        self.equals()?;
        let value = self.att_value()?;
        Ok(xml::Attr { name, value })
    }

    /// Parse an attribute value, returning the unquoted text with all
    /// references expanded.
    ///
    /// ```text
    /// AttValue  ::=  '"' ([^<&"] | Reference)* '"'
    ///             |  "'" ([^<&'] | Reference)* "'"
    /// ```
    fn att_value(&mut self) -> xml::Result<String> {
        // TODO space squishing for not CDATA
        // for now just assume CDATA
        Ok(self.expanded_text(TextMode::AttValue)?)
    }

    /// Parse a segment of text, returning the text with references expanded
    /// according to the mode:
    /// 
    /// ## `TextMode::CharData`
    /// Expands char references and entity references. Breaks on a new element
    /// (`<`).
    /// 
    /// ## `TextMode::AttValue`
    /// Expands char references and entity references. Unwraps outer quotes.
    /// 
    /// ## `TextMode::EntityValue`
    /// Expands char references and parameter entity references. Ignores entity
    /// references. Unwraps outer quotes.
    ///
    /// # Grammar
    /// ```text
    /// Content      ::=  CharData? ((Reference) CharData?)*
    /// CharData     ::=  [^<&]* - ([^<&]* ']]>' [^<&]*)
    /// AttValue     ::=  '"' ([^<&"] | Reference)* '"'
    ///                |  "'" ([^<&'] | Reference)* "'"
    /// EntityValue  ::=  '"' ([^%&"] | PEReference | Reference)* '"'
    ///                 | "'" ([^%&'] | PEReference | Reference)* "'"
    /// Reference    ::=  EntityRef | CharRef
    /// ```
    fn expanded_text(&mut self, mode: TextMode) -> xml::Result<String> {
        use TextMode::*;
        let mut buf = Vec::new();

        let delim = match mode {
            CharData => b'<',
            AttValue | EntityValue => match self.next_byte()? {
                b @ b'\'' | b @ b'"' => b,
                _ => return if mode == AttValue {
                    Err(Error::MalformedAttValue)
                } else {
                    Err(Error::MalformedEntityValue)
                }
            },
        };

        let entity_refs = match mode {
            CharData | AttValue => true,
            EntityValue => false,
        };

        let param_entity_refs = match mode {
            EntityValue => true,
            CharData | AttValue => false,
        };

        loop {
            match self.next_byte()? {
                b if b == delim => {
                    if mode == CharData {
                        // need to unget '<'
                        self.unget_byte(b);
                    }
                    break;
                }
                b'<' if mode == AttValue => return Err(Error::MalformedAttValue),
                b'&' => match self.next_byte()? {
                    b'#' => match self.next_byte()? {
                        b'x' => self.char_ref(&mut buf, CharMode::Hex)?,
                        b => {
                            self.unget_byte(b);
                            self.char_ref(&mut buf, CharMode::Decimal)?;
                        }
                    },
                    b => {
                        if entity_refs {
                            self.unget_byte(b);
                            self.entity_ref(&mut buf)?;
                        } else {
                            buf.push(b'&');
                            buf.push(b);
                        }
                    }
                },
                b'%' if param_entity_refs => {
                    return Err(Error::UnsupportedFeature(Feature::ParameterEntities))
                }
                b => {
                    buf.push(b);
                    if mode == CharData && buf.ends_with(b"]]>") {
                        return Err(Error::MalformedCharData);
                    }
                }
            }
        }

        Ok(String::from_utf8(buf)?)
    }

    /// Parse a character reference and expand the referenced value into the
    /// provided buffer.
    ///
    /// # Assumptions
    /// Assumes `&#` or `&#x` has already been read.
    ///
    /// # Grammar
    /// ```text
    /// CharRef  ::=  '&#' [0-9]+ ';'
    ///             | '&#x' [0-9a-fA-F]+ ';'
    /// ```
    fn char_ref(&mut self, buf: &mut Vec<u8>, radix: CharMode) -> xml::Result<()> {
        // XXX the subset of name chars currently includes all valid char
        // ref chars so in theory, we can just read a name and fail later
        // if that name includes invalid chars
        let name = self.name()?;
        require_byte!(b';', self, Error::MalformedCharRef);

        match u32::from_str_radix(&name, radix as u32) {
            Ok(b) => match char::from_u32(b) {
                Some(c) => {
                    let mut b = [0; 4];
                    let b = c.encode_utf8(&mut b);
                    buf.extend(b.bytes());
                    Ok(())
                }
                None => Err(Error::MalformedCharRef),
            },
            Err(_) => Err(Error::MalformedCharRef),
        }
    }

    /// Parse an entity reference and expand the referenced value
    /// into the provided buffer or prepend to the byte iter.
    ///
    /// # Assumptions
    /// Assumes `&` has already been read.
    ///
    /// # Grammar
    /// ```text
    /// EntityRef  ::=  '&' Name ';'
    /// ```
    fn entity_ref(&mut self, buf: &mut Vec<u8>) -> xml::Result<()> {
        let name = self.name()?;
        require_byte!(b';', self, Error::MalformedEntityRef);

        // predefined entities
        if let Some(b) = match name.as_ref() {
            "lt" => Some(b'<'),
            "gt" => Some(b'>'),
            "amp" => Some(b'&'),
            "apos" => Some(b'\''),
            "quot" => Some(b'"'),
            _ => None,
        } {
            buf.push(b);
            return Ok(());
        }

        #[cfg(feature = "custom-entities")]
        {
            if let Some(value) = self.custom_entities.get(name) {
                // we can't just push this text into the buffer because we need
                // to process any nested elements, so instead we add this text
                // to the front of the byte iter and continue parsing
                self.bytes.unget_buf(value.as_bytes());
                return Ok(())
            }
        }

        Err(Error::UnmappedEntityRef)
    }

    /// Parse a system literal and return the unquoted string.
    ///
    /// # Grammar
    /// ```text
    /// SystemLiteral  ::=  ('"' [^"]* '"') | ("'" [^']* "'")
    /// ```
    fn system_literal(&mut self) -> xml::Result<String> {
        let mut buf = Vec::new();

        let quote = match self.next_byte()? {
            b @ b'\'' | b @ b'"' => b,
            _ => return Err(Error::MalformedSystemLiteral),
        };

        loop {
            match self.next_byte()? {
                b if b == quote => break,
                b => buf.push(b),
            }
        }

        Ok(String::from_utf8(buf)?)
    }

    /// Parse a pubid literal and return the unquoted string.
    ///
    /// # Grammar
    /// ```text
    /// PubidLiteral  ::=  '"' PubidChar* '"' | "'" (PubidChar - "'")* "'"
    /// PubidChar     ::=  #x20 | #xD | #xA | [a-zA-Z0-9]
    ///                  | [-'()+,./:=?;!*#@$_%]
    /// ```
    fn pubid_literal(&mut self) -> xml::Result<String> {
        // TODO check characters within subset
        self.system_literal()
    }

    /// Parse a version literal and return the minor version.
    ///
    /// # Grammar
    /// ```text
    /// VersLiteral  ::=  "'" VersionNum "'" | '"' VersionNum '"'
    /// VersionNum   ::=  '1.' MinorNum
    /// MinorNum     ::= [0-9]+
    /// ```
    fn version_literal(&mut self) -> xml::Result<u32> {
        // TODO check characters within subset
        let vers = self.system_literal()?;
        if !vers.starts_with("1.") {
            return Err(Error::MalformedVersionLiteral);
        }

        match u32::from_str_radix(&vers[2..], 10) {
            Ok(v) => Ok(v),
            Err(_) => Err(Error::MalformedVersionLiteral),
        }
    }

    /// Parse an encoding literal and return the unquoted string.
    ///
    /// # Grammar
    /// ```text
    /// EncLiteral  ::=  '"' EncName '"' | "'" EncName "'"
    /// EncName     ::=  [A-Za-z] ([A-Za-z0-9._] | '-')*
    /// ```
    fn encoding_literal(&mut self) -> xml::Result<String> {
        // TODO check characters within subset
        self.system_literal()
    }

    /// YesNoLiteral  ::=  (("'" ('yes' | 'no') "'") | ('"' ('yes' | 'no') '"'))
    fn yesno_literal(&mut self) -> xml::Result<bool> {
        match self.system_literal()?.as_ref() {
            "yes" => Ok(true),
            "no" => Ok(false),
            _ => Err(Error::MalformedYesNoLiteral),
        }
    }

    /// Parse an optional external entity.
    ///
    /// # Grammar
    /// ```text
    /// ExternalID  ::=  'SYSTEM' S SystemLiteral
    ///                | 'PUBLIC' S PubidLiteral S SystemLiteral
    /// ```
    fn external_entity(&mut self) -> xml::Result<Option<ExternalEntity>> {
        let mut system = None;
        let mut pubid = None;

        match self.next_byte()? {
            b'S' => {
                require_str!("YSTEM", self, Error::MalformedExternalEntity);
                require_whitespace!(self, Error::MalformedExternalEntity);
                system = Some(self.system_literal()?);
            }
            b'P' => {
                require_str!("UBLIC", self, Error::MalformedExternalEntity);
                require_whitespace!(self, Error::MalformedExternalEntity);
                pubid = Some(self.pubid_literal()?);
                require_whitespace!(self, Error::MalformedExternalEntity);
                system = Some(self.system_literal()?);
            }
            b => self.unget_byte(b),
        }

        Ok(Some(match (system, pubid) {
            (Some(s), None) => ExternalEntity::System(s),
            (Some(s), Some(p)) => ExternalEntity::Public(p, s),
            (None, Some(_)) => return Err(Error::MalformedExternalEntity),
            (None, None) => return Ok(None),
        }))
    }
}

/// `read::Bytes` wrapper functions
impl<R: Read> Parser<R> {
    fn next_byte(&mut self) -> xml::Result<u8> {
        match self.bytes.next() {
            Some(Ok(b)) => Ok(b),
            Some(Err(e)) => Err(e.into()),
            None => Err(Error::UnexpectedEof),
        }
    }

    #[inline]
    fn read_until(&mut self, delim: u8) -> xml::Result<()> {
        while self.next_byte()? != delim {}
        Ok(())
    }

    #[inline]
    fn unget_byte(&mut self, b: u8) {
        self.bytes.unget(b)
    }

    #[inline]
    fn next_byte_is(&mut self, b: u8) -> xml::Result<bool> {
        Ok(self.next_byte()? == b)
    }
}

/// Iterater for all public tokens
impl<B: Read> Iterator for Parser<B> {
    type Item = xml::Result<Token>;

    fn next(&mut self) -> Option<xml::Result<Token>> {
        match self.token() {
            Ok(Token::EndOfFile) => None,
            t => Some(t),
        }
    }
}
