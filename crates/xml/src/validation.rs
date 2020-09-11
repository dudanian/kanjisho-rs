


/// any Unicode character, excluding the surrogate blocks, FFFE, and FFFF.
/// this is basically the equivalent of the rust `char` type minus FFFE and FFFF
/// and a lot of single bytes
/// Char ::= #x9 | #xA | #xD
///            | [#x20-#xD7FF]
///            | [#xE000-#xFFFD]
///            | [#x10000-#x10FFFF]
fn char(b: char) -> bool {
    false
}


/// NameStartChar  ::= 
///     ":" | [A-Z] | "_" | [a-z]
///       | [#xC0-#xD6] | [#xD8-#xF6]
///       | [#xF8-#x2FF] | [#x370-#x37D]
///       | [#x37F-#x1FFF] | [#x200C-#x200D]
///       | [#x2070-#x218F] | [#x2C00-#x2FEF]
///       | [#x3001-#xD7FF] | [#xF900-#xFDCF]
///       | [#xFDF0-#xFFFD] | [#x10000-#xEFFFF]
fn namestartchar(b: char) -> bool {
    false
}

/// I wonder if there is a convenient unicode set that
/// describes these ranges...
/// 
/// 
/// NameChar  ::=  NameStartChar
///                | "-" | "." | [0-9] | #xB7
///                | [#x0300-#x036F] | [#x203F-#x2040]
/// or
/// 
/// NameStartChar  ::= 
///     ":" | [A-Z] | "_" | | [a-z]
///       | "-" | "." | [0-9] | #xB7
///       | [#xC0-#xD6] | [#xD8-#xF6]
///       | [#xF8-#x37D]
///       | [#x37F-#x1FFF] | [#x200C-#x200D]
///       | [#x203F-#x2040]
///       | [#x2070-#x218F] | [#x2C00-#x2FEF]
///       | [#x3001-#xD7FF] | [#xF900-#xFDCF]
///       | [#xFDF0-#xFFFD] | [#x10000-#xEFFFF]
fn namechar(b: char) -> bool {
    false
}

/// start delimeters: < &

/// CharData  ::=  [^<&]* - ([^<&]* ']]>' [^<&]*)
fn chardata(b: char) -> bool {
    false
}