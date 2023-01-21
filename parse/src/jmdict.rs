use roxmltree::{Document, Node, ParsingOptions};

pub struct JMdict<'a> {
    doc: Document<'a>,
}

/// Entries consist of kanji elements, reading elements,
/// general information and sense elements. Each entry must have at
/// least one reading element and one sense element. Others are optional.
#[derive(Debug, Default)]
pub struct Entry {
    /// A unique numeric sequence number for each entry
    pub ent_seq: u32,
    pub k_ele: Vec<Kanji>,
    pub r_ele: Vec<Reading>,
    pub sense: Vec<Sense>,
}

/// The kanji element, or in its absence, the reading element, is
/// the defining component of each entry.
///
/// The overwhelming majority of entries will have a single kanji
/// element associated with a word in Japanese. Where there are
/// multiple kanji elements within an entry, they will be orthographical
/// variants of the same word, either using variations in okurigana, or
/// alternative and equivalent kanji. Common "mis-spellings" may be
/// included, provided they are associated with appropriate information
/// fields. Synonyms are not included; they may be indicated in the
/// cross-reference field associated with the sense element.
#[derive(Debug, Default)]
pub struct Kanji {
    /// This element will contain a word or short phrase in Japanese
    /// which is written using at least one non-kana character (usually kanji,
    /// but can be other characters). The valid characters are
    /// kanji, kana, related characters such as chouon and kurikaeshi, and
    /// in exceptional cases, letters from other alphabets.
    pub keb: String,
    /// This is a coded information field related specifically to the
    /// orthography of the keb, and will typically indicate some unusual
    /// aspect, such as okurigana irregularity.
    pub ke_inf: Vec<String>,
    /// This and the equivalent re_pri field are provided to record
    /// information about the relative priority of the entry, and consist
    /// of codes indicating the word appears in various references which
    /// can be taken as an indication of the frequency with which the word
    /// is used. This field is intended for use either by applications which
    /// want to concentrate on entries of a particular priority, or to
    /// generate subset files.
    /// 
    /// The current values in this field are:
    ///   - news1/2: appears in the "wordfreq" file compiled by Alexandre Girardi
    ///     from the Mainichi Shimbun. (See the Monash ftp archive for a copy.)
    ///     Words in the first 12,000 in that file are marked "news1" and words
    ///     in the second 12,000 are marked "news2".
    /// 
    ///   - ichi1/2: appears in the "Ichimango goi bunruishuu", Senmon Kyouiku
    ///     Publishing, Tokyo, 1998. (The entries marked "ichi2" were
    ///     demoted from ichi1 because they were observed to have low
    ///     frequencies in the WWW and newspapers.)
    /// 
    ///   - spec1 and spec2: a small number of words use this marker when they
    ///     are detected as being common, but are not included in other lists.
    /// 
    ///   - gai1/2: common loanwords, based on the wordfreq file.
    /// 
    ///   - nfxx: this is an indicator of frequency-of-use ranking in the
    ///     wordfreq file. "xx" is the number of the set of 500 words in which
    ///     the entry can be found, with "01" assigned to the first 500, "02"
    ///     to the second, and so on. (The entries with news1, ichi1, spec1, spec2
    ///     and gai1 values are marked with a "(P)" in the EDICT and EDICT2
    ///     files.)
    ///
    /// The reason both the kanji and reading elements are tagged is because
    /// on occasions a priority is only associated with a particular
    /// kanji/reading pair.
    pub ke_pri: Vec<String>,
}

/// The reading element typically contains the valid readings
/// of the word(s) in the kanji element using modern kanadzukai.
/// Where there are multiple reading elements, they will typically be
/// alternative readings of the kanji element. In the absence of a
/// kanji element, i.e. in the case of a word or phrase written
/// entirely in kana, these elements will define the entry.
#[derive(Debug, Default)]
pub struct Reading {
    /// This element content is restricted to kana and related
    /// characters such as chouon and kurikaeshi. Kana usage will be
    /// consistent between the keb and reb elements; e.g. if the keb
    /// contains katakana, so too will the reb.
    pub reb: String,
    /// This element, which will usually have a null value, indicates
    /// that the reb, while associated with the keb, cannot be regarded
    /// as a true reading of the kanji. It is typically used for words
    /// such as foreign place names, gairaigo which can be in kanji or
    /// katakana, etc.
    pub re_nokanji: bool,
    /// This element is used to indicate when the reading only applies
    /// to a subset of the keb elements in the entry. In its absence, all
    /// readings apply to all kanji elements. The contents of this element
    /// must exactly match those of one of the keb elements.
    pub re_restr: Vec<String>,
    /// General coded information pertaining to the specific reading.
    /// Typically it will be used to indicate some unusual aspect of
    /// the reading.
    pub re_inf: Vec<String>,
    /// See the comment on ke_pri above.
    pub re_pri: Vec<String>,
}

/// The sense element will record the translational equivalent
/// of the Japanese word, plus other related information. Where there
/// are several distinctly different meanings of the word, multiple
/// sense elements will be employed.
#[derive(Debug, Default)]
pub struct Sense {
    /// This element, if present, indicate that the sense is restricted
    /// to the lexeme represented by the keb.
    pub stagk: Vec<String>,
    /// This element, if present, indicate that the sense is restricted
    /// to the lexeme represented by the reb.
    pub stagr: Vec<String>,
    /// Part-of-speech information about the entry/sense. Should use
    /// appropriate entity codes. In general where there are multiple senses
    /// in an entry, the part-of-speech of an earlier sense will apply to
    /// later senses unless there is a new part-of-speech indicated.
    pub pos: Vec<String>,
    /// This element is used to indicate a cross-reference to another
    /// entry with a similar or related meaning or sense. The content of
    /// this element is typically a keb or reb element in another entry. In some
    /// cases a keb will be followed by a reb and/or a sense number to provide
    /// a precise target for the cross-reference. Where this happens, a JIS
    /// "centre-dot" (0x2126) is placed between the components of the
    /// cross-reference. The target keb or reb must not contain a centre-dot.
    pub xref: Vec<String>,
    /// This element is used to indicate another entry which is an
    /// antonym of the current entry/sense. The content of this element
    /// must exactly match that of a keb or reb element in another entry.
    pub ant: Vec<String>,
    /// Information about the field of application of the entry/sense.
    /// When absent, general application is implied. Entity coding for
    /// specific fields of application.
    pub field: Vec<String>,
    /// This element is used for other relevant information about
    /// the entry/sense. As with part-of-speech, information will usually
    /// apply to several senses.
    pub misc: Vec<String>,
    /// The sense-information elements provided for additional
    /// information to be recorded about a sense. Typical usage would
    /// be to indicate such things as level of currency of a sense, the
    /// regional variations, etc.
    pub s_inf: Vec<String>,
    pub lsource: Vec<Lang>,
    /// For words specifically associated with regional dialects in
    /// Japanese, the entity code for that dialect, e.g. ksb for Kansaiben.
    pub dial: Vec<String>,
    pub gloss: Vec<Gloss>,
}

/// This element records the information about the source
/// language(s) of a loan-word/gairaigo. If the source language is other
/// than English, the language is indicated by the xml:lang attribute.
/// The element value (if any) is the source word or phrase.
#[derive(Debug, Default)]
pub struct Lang {
    pub lsource: String,
    /// The xml:lang attribute defines the language(s) from which
    /// a loanword is drawn. It will be coded using the three-letter language
    /// code from the ISO 639-2 standard. When absent, the value "eng" (i.e.
    /// English) is the default value. The bibliographic (B) codes are used.
    pub lang: String,
    /// The ls_type attribute indicates whether the lsource element
    /// fully or partially describes the source word or phrase of the
    /// loanword. If absent, it will have the implied value of "full".
    /// Otherwise it will contain "part".
    pub ls_type: bool,
    /// The ls_wasei attribute indicates that the Japanese word
    /// has been constructed from words in the source language, and
    /// not from an actual phrase in that language. Most commonly used to
    /// indicate "waseieigo".
    pub ls_wasei: bool,
}

/// Within each sense will be one or more "glosses", i.e.
/// target-language words or phrases which are equivalents to the
/// Japanese word. This element would normally be present, however it
/// may be omitted in entries which are purely for a cross-reference.
#[derive(Debug, Default)]
pub struct Gloss {
    pub gloss: String,
    /// The xml:lang attribute defines the target language of the
    /// gloss. It will be coded using the three-letter language code from
    /// the ISO 639 standard. When absent, the value "eng" (i.e. English)
    /// is the default value.
    pub lang: String,
    /// The g_type attribute specifies that the gloss is of a particular
    /// type, e.g. "lit" (literal), "fig" (figurative), "expl" (explanation).
    pub g_type: Option<String>,
}



impl<'a> JMdict<'a> {
    pub fn entries(self: &'a Self) -> impl Iterator<Item = Entry> + 'a {
        return self
            .doc
            .root_element()
            .children()
            .filter(|n| n.is_element())
            .map(parse_entry);
    }
}


pub fn parse<'a>(text: &'a str) -> JMdict<'a> {
    let opt = ParsingOptions { allow_dtd: true };
    let doc = Document::parse_with_options(text, opt).expect("failed to parse");

    return JMdict { doc };
}

fn parse_entry(node: Node) -> Entry {
    let mut e = Entry::default();

    for n in node.children().filter(|n| n.is_element()) {
        match n.tag_name().name() {
            "ent_seq" => e.ent_seq = get_num(n.text()),
            "k_ele" => e.k_ele.push(parse_k_ele(n)),

            tag => println!("Warning: unexpected tag name {}", tag),
        }
    }

    e
}


fn parse_k_ele(node: Node) -> Kanji {
    for n in node.children().filter(|n| n.is_element()) {
        match n.tag_name().name() {
            "grade" => entry.grade = Some(get_num(n.text())),
            "stroke_count" => entry.stroke_count.push(get_num(n.text())),
            "variant" => entry.variant.push(Variant {
                variant: get_text(n.text()),
                var_type: get_text(n.attribute("var_type")),
            }),
            "freq" => entry.freq = Some(get_num(n.text())),
            "rad_name" => entry.rad_name.push(get_text(n.text())),
            "jlpt" => entry.jlpt = Some(get_num(n.text())),
            tag => println!("Warning: unexpected tag name in misc: {}", tag),
        };
    }
}


// TODO these should probably all be falliable
fn get_text(s: Option<&str>) -> String {
    get_optional_text(s).expect("no text")
}

fn get_num(s: Option<&str>) -> u32 {
    get_optional_num(s).expect("no number")
}

fn get_optional_text(s: Option<&str>) -> Option<String> {
    s.map(|s| s.trim().into())
}

fn get_optional_num(s: Option<&str>) -> Option<u32> {
    s.map(|s| s.trim().parse().expect("failed to parse"))
}