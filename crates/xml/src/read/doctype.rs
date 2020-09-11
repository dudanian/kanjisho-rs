//! Doctype specific parsing
//!
//! Parsing doctypes is like parsing another language on top of xml. This module
//! is more like an extension to the `read` module, since we are continuing the
//! `Parser` impl
use super::*;

// DOCTYPE parsing
impl<R: Read> Parser<R> {
    /// Parse a doctype token from an internal subset.
    ///
    /// # Grammar
    /// ```text
    /// intSubset    ::=  (markupdecl | DeclSep)* InternalEnd
    /// markupdecl   ::=  elementdecl | AttlistDecl | EntityDecl | NotationDecl
    ///                 | PI | Comment
    /// DeclSep      ::=  PEReference | S
    /// InternalEnd  ::=  ']' S? '>'
    /// ```
    pub(super) fn doctype_token(&mut self) -> xml::Result<RawToken> {
        use super::Doctype::*;

        match self.next_byte()? {
            b'<' => match self.next_byte()? {
                b'?' => self.proc_inst(),
                b'!' => match self.next_byte()? {
                    b'-' => self.comment(),
                    b => {
                        self.unget_byte(b);
                        match self.name()?.as_ref() {
                            "ENTITY" => self.entity_decl(),
                            "ELEMENT" => self.element_decl(),
                            "ATTLIST" => self.attlist_decl(),
                            "NOTATION" => self.notation_decl(),
                            _ => Err(Error::MalformedDoctype),
                        }
                    }
                },
                _ => return Err(Error::MalformedDoctype),
            },
            b']' => {
                self.whitespace()?;
                require_byte!(b'>', self, Error::MalformedDoctype);
                Ok(RawToken::DoctypeDef(InternalEnd))
            }
            b'%' => self.param_entity_ref(),
            _ => Err(Error::MalformedDoctype),
        }
    }

    /// Parse an EntityDecl element.
    ///
    /// # Assumptions
    /// Assumes that `<!ENTITY` has already been read.
    /// 
    /// # Grammar
    /// ```text
    /// EntityDecl  ::=  GEDecl | PEDecl
    /// GEDecl      ::=  '<!ENTITY' S Name S EntityDef S? '>'
    /// PEDecl      ::=  '<!ENTITY' S '%' S Name S PEDef S? '>'
    /// EntityDef   ::=  EntityValue | (ExternalID NDataDecl?)
    /// PEDef       ::=  EntityValue | ExternalID
    /// NDataDecl   ::=  S 'NDATA' S Name
    /// ```
    fn entity_decl(&mut self) -> xml::Result<RawToken> {
        require_whitespace!(self, Error::MalformedEntityDecl);
        if self.next_byte_is(b'%')? {
            return Err(Error::UnsupportedFeature(Feature::ParameterEntities));
        }

        let name = self.name()?;
        require_whitespace!(self, Error::MalformedEntityDecl);
        if let Some(_) = self.external_entity()? {
            return Err(Error::UnsupportedFeature(Feature::ExternalEntities));
        }

        let value = self.entity_value()?;
        // TODO insert entity into custom entities map

        self.whitespace()?;
        require_byte!(b'>', self, Error::MalformedEntityDecl);
        Ok(RawToken::DoctypeDef(Doctype::EntityDecl))
    }

    /// Parse an ElementDecl element.
    ///
    /// # Assumptions
    /// Assumes that `<!ELEMENT` has already been read.
    /// 
    /// # Grammar
    /// ```text
    /// elementdecl  ::=  '<!ELEMENT' S Name S contentspec S? '>'
    /// contentspec  ::=  'EMPTY' | 'ANY' | Mixed | children
    /// Mixed        ::=  '(' S? '#PCDATA' (S? '|' S? Name)* S? ')*'
    ///                 | '(' S? '#PCDATA' S? ')'
    /// children     ::=  (choice | seq) ('?' | '*' | '+')?
    /// cp           ::=  (Name | choice | seq) ('?' | '*' | '+')?
    /// choice       ::=  '(' S? cp ( S? '|' S? cp )+ S? ')'
    /// seq          ::=  '(' S? cp ( S? ',' S? cp )* S? ')'
    /// ```
    fn element_decl(&mut self) -> xml::Result<RawToken> {
        require_whitespace!(self, Error::MalformedEntityDecl);
        let name = self.name()?;
        require_whitespace!(self, Error::MalformedEntityDecl);

        // TODO actually parse elementdecl
        self.read_until(b'>')?;
        Ok(RawToken::DoctypeDef(Doctype::ElementDecl))
    }

    /// Parse an AttlistDecl element.
    ///
    /// # Assumptions
    /// Assumes that `<!ATTLIST` has already been read.
    /// 
    /// # Grammar
    /// ```text
    /// AttlistDecl     ::=  '<!ATTLIST' S Name AttDef* S? '>'
    /// AttDef          ::=  S Name S AttType S DefaultDecl
    /// AttType         ::=  StringType | TokenizedType | EnumeratedType
    /// StringType      ::=  'CDATA'
    /// TokenizedType   ::=  'ID'
    ///                    | 'IDREF'
    ///                    | 'IDREFS'
    ///                    | 'ENTITY'
    ///                    | 'ENTITIES'
    ///                    | 'NMTOKEN'
    ///                    | 'NMTOKENS'
    /// EnumeratedType  ::=  NotationType | Enumeration
    /// NotationType    ::=  'NOTATION' S '(' S? Name (S? '|' S? Name)* S? ')'
    /// Enumeration     ::=  '(' S? Nmtoken (S? '|' S? Nmtoken)* S? ')'
    /// DefaultDecl     ::=  '#REQUIRED' | '#IMPLIED'
    ///                    | (('#FIXED' S)? AttValue)
    /// ```
    fn attlist_decl(&mut self) -> xml::Result<RawToken> {
        require_whitespace!(self, Error::MalformedAttlistDecl);
        let name = self.name()?;

        // TODO actually parse attlist
        self.read_until(b'>')?;
        Ok(RawToken::DoctypeDef(Doctype::AttlistDecl))
    }

    /// Parse a NotationDecl element.
    ///
    /// # Assumptions
    /// Assumes that `<!NOTATION` has already been read.
    /// 
    /// # Grammar
    /// ```text
    /// NotationDecl  ::=  '<!NOTATION' S Name S (ExternalID | PublicID) S? '>'
    /// PublicID      ::=  'PUBLIC' S PubidLiteral
    /// ```
    fn notation_decl(&mut self) -> xml::Result<RawToken> {
        // TODO possibly support notations
        Err(Error::UnsupportedFeature(Feature::Notations))
    }

    /// Parse a Parameter Entity reference.
    ///
    /// # Assumptions
    /// Assumes that `%` has already been read.
    /// 
    /// # Grammar
    /// ```text
    /// PEReference  ::=  '%' Name ';'
    /// ```
    fn param_entity_ref(&mut self) -> xml::Result<RawToken> {
        // TODO possibly support parameter entity references
        Err(Error::UnsupportedFeature(Feature::ParameterEntities))
    }

    /// Parse an entity value and returns the unquoted expanded string.
    /// 
    /// # Grammar
    /// ```text
    /// EntityValue  ::=  '"' ([^%&"] | PEReference | Reference)* '"'
    ///                 | "'" ([^%&'] | PEReference | Reference)* "'"
    fn entity_value(&mut self) -> xml::Result<String> {
        let mut buf = Vec::new();

        let quote = match self.next_byte()? {
            b @ b'\'' | b @ b'"' => b,
            _ => return Err(Error::MalformedEntityValue),
        };

        loop {
            match self.next_byte()? {
                b'%' => return Err(Error::UnsupportedFeature(Feature::ParameterEntities)),
                b'&' => match self.next_byte()? {
                    b'#' => match self.next_byte()? {
                        b'x' => self.char_ref(&mut buf, CharMode::Hex)?,
                        b => {
                            self.unget_byte(b);
                            self.char_ref(&mut buf, CharMode::Decimal)?;
                        }
                    },
                    b => {
                        // XXX technically, expanding entity refs here is
                        // non-compliant but I don't have a convenient way to
                        // expand and reparse these at the moment
                        self.unget_byte(b);
                        self.entity_ref(&mut buf)?;
                    }
                },
                b if b == quote => break,
                b => buf.push(b),
            }
        }

        Ok(String::from_utf8(buf)?)
    }
}
