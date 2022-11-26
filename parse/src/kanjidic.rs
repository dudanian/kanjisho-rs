use roxmltree::{Document, Node, ParsingOptions};

/// This module is based off of the DTD of the XML-format kanji file
/// combining information from the KANJIDIC and KANJD212 files. This
/// struct aims to reproduce the KANJIDIC format to the fullest with
/// no assumptions about how the data will be used.
///
/// The Kanjidic covers the following kanji:
///  * the 6,355 kanji from JIS X 0208;
///  * the 5,801 kanji from JIS X 0212;
///  * the 3,693 kanji from JIS X 0213 as follows:
///    * the 2,741 kanji which are also in JIS X 0212 have
///      JIS X 0213 code-points (kuten) added to the existing entry;
///    * the 952 "new" kanji have new entries.
///
/// At the end of the explanation for a number of fields there is a tag
/// with the format [N]. This indicates the leading letter(s) of the
/// equivalent field in the KANJIDIC and KANJD212 files.
///
/// The KANJIDIC documentation should also be read for additional
/// information about the information in the file.
pub struct Kanjidic<'a> {
    doc: Document<'a>,
}

/// The single header element will contain identification information
/// about the version of the file
#[derive(Debug, Default)]
pub struct Header {
    /// This field denotes the version of kanjidic2 structure, as more
    /// than one version may exist.
    file_version: u32,
    /// The version of the file, in the format YYYY-NN, where NN will be
    /// a number starting with 01 for the first version released in a
    /// calendar year, then increasing for each version in that year.
    database_version: String,
    /// The date the file was created in international format (YYYY-MM-DD).
    date_of_creation: String,
}

/// A Kanji entry in the dictionary
#[derive(Debug, Default)]
pub struct Kanji {
    /// The character itself in UTF8 coding.
    pub literal: char,
    /// The kanji grade level. 1 through 6 indicates a Kyouiku kanji
    /// and the grade in which the kanji is taught in Japanese schools.
    /// 8 indicates it is one of the remaining Jouyou Kanji to be learned
    /// in junior high school. 9 indicates it is a Jinmeiyou (for use
    /// in names) kanji which in addition to the Jouyou kanji are approved
    /// for use in family name registers and other official documents. 10
    /// also indicates a Jinmeiyou kanji which is a variant of a
    /// Jouyou kanji. [G]
    pub grade: Option<u32>,
    /// The stroke count of the kanji, including the radical. If more than
    /// one, the first is considered the accepted count, while subsequent ones
    /// are common miscounts. (See Appendix E. of the KANJIDIC documentation
    /// for some of the rules applied when counting strokes in some of the
    /// radicals.) [S]
    pub stroke_count: Vec<u32>,
    /// A frequency-of-use ranking. The 2,500 most-used characters have a
    /// ranking; those characters that lack this field are not ranked. The
    /// frequency is a number from 1 to 2,500 that expresses the relative
    /// frequency of occurrence of a character in modern Japanese. This is
    /// based on a survey in newspapers, so it is biassed towards kanji
    /// used in newspaper articles. The discrimination between the less
    /// frequently used kanji is not strong. (Actually there are 2,501
    /// kanji ranked as there was a tie.)
    pub freq: Option<u32>,
    /// When the kanji is itself a radical and has a name, this element
    /// contains the name (in hiragana.) [T2]
    pub rad_name: Vec<String>,
    /// The (former) Japanese Language Proficiency test level for this kanji.
    /// Values range from 1 (most advanced) to 4 (most elementary). This field
    /// does not appear for kanji that were not required for any JLPT level.
    /// Note that the JLPT test levels changed in 2010, with a new 5-level
    /// system (N1 to N5) being introduced. No official kanji lists are
    /// available for the new levels. The new levels are regarded as
    /// being similar to the old levels except that the old level 2 is
    /// now divided between N2 and N3.
    pub jlpt: Option<u32>,
    /// Japanese readings that are now only associated with names.
    pub nanori: Vec<String>,

    // remaining nested fields
    pub codepoint: Vec<Codepoint>,
    pub radical: Vec<Radical>,
    pub variant: Vec<Variant>,
    pub dic_number: Vec<DicRef>,
    pub quecy_code: Vec<QueryCode>,
    pub rmgroup: Vec<ReadingMeaning>,
}

/// The codepoint element states the code of the character in the various
/// character set standards.
#[derive(Debug, Default)]
pub struct Codepoint {
    /// The cp_value contains the codepoint of the character in a particular
    /// standard. The standard will be identified in the cp_type attribute.
    pub cp_value: String,
    /// The cp_type attribute states the coding standard applying to the
    /// element. The values assigned so far are:
    ///  * jis208 - JIS X 0208-1997 - kuten coding (nn-nn)
    ///  * jis212 - JIS X 0212-1990 - kuten coding (nn-nn)
    ///  * jis213 - JIS X 0213-2000 - kuten coding (p-nn-nn)
    ///  * ucs - Unicode 4.0 - hex coding (4 or 5 hexadecimal digits)
    pub cp_type: String,
}

#[derive(Debug, Default)]
pub struct Radical {
    /// The radical number, in the range 1 to 214. The particular
    /// classification type is stated in the rad_type attribute.
    pub rad_value: u32,
    /// The rad_type attribute states the type of radical classification.
    ///  * classical - based on the system first used in the KangXi Zidian.
    ///      The Shibano "JIS Kanwa Jiten" is used as the reference source.
    ///  * nelson_c - as used in the Nelson "Modern Japanese-English
    ///      Character Dictionary" (i.e. the Classic, not the New Nelson).
    ///      This will only be used where Nelson reclassified the kanji.
    pub rad_type: String,
}

/// Either a cross-reference code to another kanji, usually regarded as a
/// variant, or an alternative indexing code for the current kanji.
/// The type of variant is given in the var_type attribute.
#[derive(Debug, Default)]
pub struct Variant {
    pub variant: String,
    /// The var_type attribute indicates the type of variant code. The current
    /// values are:
    ///  * jis208 - in JIS X 0208 - kuten coding
    ///  * jis212 - in JIS X 0212 - kuten coding
    ///  * jis213 - in JIS X 0213 - kuten coding
    ///      (most of the above relate to "shinjitai/kyuujitai"
    ///      alternative character glyphs)
    ///  * deroo - De Roo number - numeric
    ///  * njecd - Halpern NJECD index number - numeric
    ///  * s_h - The Kanji Dictionary (Spahn & Hadamitzky) - descriptor
    ///  * nelson_c - "Classic" Nelson - numeric
    ///  * oneill - Japanese Names (O'Neill) - numeric
    ///  * ucs - Unicode codepoint- hex
    pub var_type: String,
}

/// This element contains the index numbers and similar unstructured
/// information such as page numbers in a number of published dictionaries,
/// and instructional books on kanji.
#[derive(Debug, Default)]
pub struct DicRef {
    /// Each dic_ref contains an index number. The particular dictionary,
    /// etc. is defined by the dr_type attribute.
    /// Note: this field is not always a number.
    pub dic_ref: String,
    /// The dr_type defines the dictionary or reference book, etc. to which
    /// dic_ref element applies. The initial allocation is:
    ///  * nelson_c - "Modern Reader's Japanese-English Character Dictionary",
    ///      edited by Andrew Nelson (now published as the "Classic"
    ///      Nelson).
    ///  * nelson_n - "The New Nelson Japanese-English Character Dictionary",
    ///      edited by John Haig.
    ///  * halpern_njecd - "New Japanese-English Character Dictionary",
    ///      edited by Jack Halpern.
    ///  * halpern_kkd - "Kodansha Kanji Dictionary", (2nd Ed. of the NJECD)
    ///      edited by Jack Halpern.
    ///  * halpern_kkld - "Kanji Learners Dictionary" (Kodansha) edited by
    ///      Jack Halpern.
    ///  * halpern_kkld_2ed - "Kanji Learners Dictionary" (Kodansha), 2nd edition
    ///      (2013) edited by Jack Halpern.
    ///  * heisig - "Remembering The Kanji" by James Heisig.
    ///  * heisig6 - "Remembering The Kanji, Sixth Ed." by James Heisig.
    ///  * gakken - "A New Dictionary of Kanji Usage" (Gakken)
    ///  * oneill_names - "Japanese Names", by P.G. O'Neill.
    ///  * oneill_kk - "Essential Kanji" by P.G. O'Neill.
    ///  * moro - "Daikanwajiten" compiled by Morohashi. For some kanji two
    ///      additional attributes are used: m_vol: the volume of the
    ///      dictionary in which the kanji is found, and m_page: the page
    ///      number in the volume.
    ///  * henshall - "A Guide To Remembering Japanese Characters" by
    ///      Kenneth G. Henshall.
    ///  * sh_kk - "Kanji and Kana" by Spahn and Hadamitzky.
    ///  * sh_kk2 - "Kanji and Kana" by Spahn and Hadamitzky (2011 edition).
    ///  * sakade - "A Guide To Reading and Writing Japanese" edited by
    ///      Florence Sakade.
    ///  * jf_cards - Japanese Kanji Flashcards, by Max Hodges and
    ///      Tomoko Okazaki. (Series 1)
    ///  * henshall3 - "A Guide To Reading and Writing Japanese" 3rd
    ///      edition, edited by Henshall, Seeley and De Groot.
    ///  * tutt_cards - Tuttle Kanji Cards, compiled by Alexander Kask.
    ///  * crowley - "The Kanji Way to Japanese Language Power" by
    ///      Dale Crowley.
    ///  * kanji_in_context - "Kanji in Context" by Nishiguchi and Kono.
    ///  * busy_people - "Japanese For Busy People" vols I-III, published
    ///      by the AJLT. The codes are the volume.chapter.
    ///  * kodansha_compact - the "Kodansha Compact Kanji Guide".
    ///  * maniette - codes from Yves Maniette's "Les Kanjis dans la tete"
    ///      French adaptation of Heisig.
    pub dr_type: String,
    /// See above under "moro".
    pub m_vol: Option<u32>,
    /// See above under "moro".
    pub m_page: Option<u32>,
}

/// These codes contain information relating to the glyph, and can be used
/// for finding a required kanji. The type of code is defined by the
/// qc_type attribute.
#[derive(Debug, Default)]
pub struct QueryCode {
    /// The q_code contains the actual query-code value, according to the
    /// qc_type attribute.
    pub q_code: String,
    /// The qc_type attribute defines the type of query code. The current values
    /// are:
    ///  * skip - Halpern's SKIP (System of Kanji Indexing by Patterns)
    ///      code. The format is n-nn-nn. See the KANJIDIC documentation
    ///      for a description of the code and restrictions on the
    ///      commercial use of this data. [P] There are also
    ///      a number of misclassification codes, indicated by the
    ///      "skip_misclass" attribute.
    ///  * sh_desc - the descriptor codes for The Kanji Dictionary (Tuttle
    ///      1996) by Spahn and Hadamitzky. They are in the form nxnn.n,
    ///      e.g. 3k11.2, where the kanji has 3 strokes in the
    ///      identifying radical, it is radical "k" in the SH
    ///      classification system, there are 11 other strokes, and it is
    ///      the 2nd kanji in the 3k11 sequence. (I am very grateful to
    ///      Mark Spahn for providing the list of these descriptor codes
    ///      for the kanji in this file.) [I]
    ///  * four_corner - the "Four Corner" code for the kanji. This is a code
    ///      invented by Wang Chen in 1928. See the KANJIDIC documentation
    ///      for an overview of the Four Corner System. [Q]
    ///  * deroo - the codes developed by the late Father Joseph De Roo, and
    ///      published in his book "2001 Kanji" (Bonjinsha). Fr De Roo
    ///      gave his permission for these codes to be included. [DR]
    ///  * misclass - a possible misclassification of the kanji according
    ///      to one of the code types. (See the "Z" codes in the KANJIDIC
    ///      documentation for more details.)
    pub qc_type: String,
    /// The values of this attribute indicate the type if
    /// misclassification:
    ///  * posn - a mistake in the division of the kanji
    ///  * stroke_count - a mistake in the number of strokes
    ///  * stroke_and_posn - mistakes in both division and strokes
    ///  * stroke_diff - ambiguous stroke counts depending on glyph
    pub skip_misclass: Option<String>,
}

/// The readings for the kanji in several languages, and the meanings, also
/// in several languages. The readings and meanings are grouped to enable
/// the handling of the situation where the meaning is differentiated by
/// reading. [T1]
#[derive(Debug, Default)]
pub struct ReadingMeaning {
    pub reading: Vec<Reading>,
    pub meaning: Vec<Meaning>,
}

#[derive(Debug, Default)]
pub struct Reading {
    /// The reading element contains the reading or pronunciation
    /// of the kanji.
    pub reading: String,
    /// The r_type attribute defines the type of reading in the reading
    /// element. The current values are:
    ///  * pinyin - the modern PinYin romanization of the Chinese reading
    ///      of the kanji. The tones are represented by a concluding
    ///      digit. [Y]
    ///  * korean_r - the romanized form of the Korean reading(s) of the
    ///      kanji. The readings are in the (Republic of Korea) Ministry
    ///      of Education style of romanization. [W]
    ///  * korean_h - the Korean reading(s) of the kanji in hangul.
    ///  * vietnam - the Vietnamese readings supplied by Minh Chau Pham.
    ///  * ja_on - the "on" Japanese reading of the kanji, in katakana.
    ///  * ja_kun - the "kun" Japanese reading of the kanji, usually in
    ///      hiragana.
    ///      Where relevant the okurigana is also included separated by a
    ///      ".". Readings associated with prefixes and suffixes are
    ///      marked with a "-".
    pub r_type: String,
}

#[derive(Debug, Default)]
pub struct Meaning {
    /// The meaning associated with the kanji.
    pub meaning: String,
    /// The m_lang attribute defines the target language of the meaning. It
    /// will be coded using the two-letter language code from the ISO 639-1
    /// standard. When absent, the value "en" (i.e. English) is implied.
    pub m_lang: String,
}

impl<'a> Kanjidic<'a> {
    pub fn header(self: &Self) -> Header {
        let mut h = Header::default();

        let node = self
            .doc
            .root_element()
            .children()
            .filter(|n| n.is_element())
            .next()
            .expect("no header node");

        for n in node.children() {
            match n.tag_name().name() {
                "file_version" => h.file_version = get_num(n.text()),
                "database_version" => h.database_version = get_text(n.text()),
                "date_of_creation" => h.date_of_creation = get_text(n.text()),
                _ => (),
            }
        }

        return h;
    }

    pub fn entries(self: &'a Self) -> impl Iterator<Item = Kanji> + 'a {
        return self
            .doc
            .root_element()
            .children()
            .filter(|n| n.is_element())
            // first element is the header
            .skip(1)
            .map(parse_entry);
    }
}

pub fn parse<'a>(text: &'a str) -> Kanjidic<'a> {
    let opt = ParsingOptions { allow_dtd: true };
    let doc = Document::parse_with_options(text, opt).expect("failed to parse");

    return Kanjidic { doc };
}

fn parse_entry(node: Node) -> Kanji {
    let mut k = Kanji::default();

    for n in node.children().filter(|n| n.is_element()) {
        match n.tag_name().name() {
            "literal" => parse_literal(n, &mut k),
            "codepoint" => parse_codepoint(n, &mut k),
            "radical" => parse_radical(n, &mut k),
            "misc" => parse_misc(n, &mut k),
            "dic_number" => parse_dic_number(n, &mut k),
            "query_code" => parse_query_code(n, &mut k),
            "reading_meaning" => parse_reading_meaning(n, &mut k),
            tag => println!("Warning: unexpected tag name {}", tag),
        }
    }

    k
}

fn parse_literal(node: Node, entry: &mut Kanji) {
    let text = node.text().expect("failed to get text").trim();
    let c = text.chars().next().expect("no first char");
    entry.literal = c
}

fn parse_codepoint(node: Node, entry: &mut Kanji) {
    entry.codepoint = node
        .children()
        .filter(|n| n.is_element())
        .map(|n| Codepoint {
            cp_value: get_text(n.text()),
            cp_type: get_text(n.attribute("cp_type")),
        })
        .collect();
}

fn parse_radical(node: Node, entry: &mut Kanji) {
    entry.radical = node
        .children()
        .filter(|n| n.is_element())
        .map(|n| Radical {
            rad_value: get_num(n.text()),
            rad_type: get_text(n.attribute("rad_type")),
        })
        .collect();
}

fn parse_misc(node: Node, entry: &mut Kanji) {
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

fn parse_dic_number(node: Node, entry: &mut Kanji) {
    entry.dic_number = node
        .children()
        .filter(|n| n.is_element())
        .map(|n| DicRef {
            dic_ref: get_text(n.text()),
            dr_type: get_text(n.attribute("dr_type")),
            m_vol: get_optional_num(n.attribute("m_vol")),
            m_page: get_optional_num(n.attribute("m_page")),
        })
        .collect();
}

fn parse_query_code(node: Node, entry: &mut Kanji) {
    entry.quecy_code = node
        .children()
        .filter(|n| n.is_element())
        .map(|n| QueryCode {
            q_code: get_text(n.text()),
            qc_type: get_text(n.attribute("qc_type")),
            skip_misclass: get_optional_text(n.attribute("skip_misclass")),
        })
        .collect();
}

fn parse_reading_meaning(node: Node, entry: &mut Kanji) {
    for n in node.children().filter(|n| n.is_element()) {
        match n.tag_name().name() {
            "rmgroup" => parse_rmgroup(n, entry),
            "nanori" => entry.nanori.push(get_text(n.text())),
            tag => println!("Warning: unexpected tag name in reading_meaning: {}", tag),
        };
    }
}

fn parse_rmgroup(node: Node, entry: &mut Kanji) {
    let mut group = ReadingMeaning::default();

    for n in node.children().filter(|n| n.is_element()) {
        match n.tag_name().name() {
            "reading" => group.reading.push(Reading {
                reading: get_text(n.text()),
                r_type: get_text(n.attribute("r_type")),
            }),
            "meaning" => group.meaning.push(Meaning {
                meaning: get_text(n.text()),
                m_lang: get_optional_text(n.attribute("m_lang")).unwrap_or("en".into()),
            }),
            tag => println!("Warning: unexpected tag name in reading_meaning: {}", tag),
        };
    }
    entry.rmgroup.push(group);
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

#[test]
fn test_parse() {
    let text = super::read_file("kanjidic2.xml");
    for kanji in parse(&text).entries() {
        println!("{:?}", kanji);
    }
}
