use roxmltree::{Document, Node, ParsingOptions};
use serde::{Deserialize, Serialize};

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
/// with the format \[N\]. This indicates the leading letter(s) of the
/// equivalent field in the KANJIDIC and KANJD212 files.
///
/// The KANJIDIC documentation should also be read for additional
/// information about the information in the file.
pub struct Kanjidic<'a> {
    doc: Document<'a>,
}

/// The single header element will contain identification information
/// about the version of the file
#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Header {
    /// This field denotes the version of kanjidic2 structure, as more
    /// than one version may exist.
    file_version: i32,
    /// The version of the file, in the format YYYY-NN, where NN will be
    /// a number starting with 01 for the first version released in a
    /// calendar year, then increasing for each version in that year.
    database_version: String,
    /// The date the file was created in international format (YYYY-MM-DD).
    date_of_creation: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum NumString {
    Num(i32),
    String(String),
}

/// A Kanji entry in the dictionary
#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Kanji {
    /// The character itself in UTF8 coding.
    pub literal: char,
    /// The codepoint element states the code of the character in the various
    /// character set standards.
    ///
    /// The cp_value contains the codepoint of the character in a particular
    /// standard. The standard will be identified in the cp_type attribute.
    ///
    /// The cp_type attribute states the coding standard applying to the
    /// element. The values assigned so far are:
    ///  * jis208 - JIS X 0208-1997 - kuten coding (nn-nn)
    ///  * jis212 - JIS X 0212-1990 - kuten coding (nn-nn)
    ///  * jis213 - JIS X 0213-2000 - kuten coding (p-nn-nn)
    ///  * ucs - Unicode 4.0 - hex coding (4 or 5 hexadecimal digits)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub codepoint: Vec<Reference>,
    /// The radical number, in the range 1 to 214. The particular
    /// classification type is stated in the rad_type attribute.
    ///
    /// The rad_type attribute states the type of radical classification.
    ///  * classical - based on the system first used in the KangXi Zidian.
    ///      The Shibano "JIS Kanwa Jiten" is used as the reference source.
    ///  * nelson_c - as used in the Nelson "Modern Japanese-English
    ///      Character Dictionary" (i.e. the Classic, not the New Nelson).
    ///      This will only be used where Nelson reclassified the kanji.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub radical: Vec<Reference<i32>>,
    /// The kanji grade level. 1 through 6 indicates a Kyouiku kanji
    /// and the grade in which the kanji is taught in Japanese schools.
    /// 8 indicates it is one of the remaining Jouyou Kanji to be learned
    /// in junior high school. 9 indicates it is a Jinmeiyou (for use
    /// in names) kanji which in addition to the Jouyou kanji are approved
    /// for use in family name registers and other official documents. 10
    /// also indicates a Jinmeiyou kanji which is a variant of a
    /// Jouyou kanji. [G]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grade: Option<i32>,
    /// The stroke count of the kanji, including the radical.
    pub stroke_count: i32,
    /// Common stroke miscounts. (See Appendix E. of the KANJIDIC documentation
    /// for some of the rules applied when counting strokes in some of the
    /// radicals.) [S]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stroke_miscounts: Vec<i32>,
    /// Either a cross-reference code to another kanji, usually regarded as a
    /// variant, or an alternative indexing code for the current kanji.
    /// The type of variant is given in the var_type attribute.
    ///
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variant: Vec<Reference>,
    /// A frequency-of-use ranking. The 2,500 most-used characters have a
    /// ranking; those characters that lack this field are not ranked. The
    /// frequency is a number from 1 to 2,500 that expresses the relative
    /// frequency of occurrence of a character in modern Japanese. This is
    /// based on a survey in newspapers, so it is biassed towards kanji
    /// used in newspaper articles. The discrimination between the less
    /// frequently used kanji is not strong. (Actually there are 2,501
    /// kanji ranked as there was a tie.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub freq: Option<i32>,
    /// When the kanji is itself a radical and has a name, this element
    /// contains the name (in hiragana.) [T2]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub names: Vec<String>,
    /// The (former) Japanese Language Proficiency test level for this kanji.
    /// Values range from 1 (most advanced) to 4 (most elementary). This field
    /// does not appear for kanji that were not required for any JLPT level.
    /// Note that the JLPT test levels changed in 2010, with a new 5-level
    /// system (N1 to N5) being introduced. No official kanji lists are
    /// available for the new levels. The new levels are regarded as
    /// being similar to the old levels except that the old level 2 is
    /// now divided between N2 and N3.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jlpt: Option<i32>,
    /// This element contains the index numbers and similar unstructured
    /// information such as page numbers in a number of published dictionaries,
    /// and instructional books on kanji.
    ///
    /// Each dic_ref contains an index number. The particular dictionary,
    /// etc. is defined by the dr_type attribute.
    ///
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dict: Vec<Reference<NumString>>,
    /// These codes contain information relating to the glyph, and can be used
    /// for finding a required kanji. The type of code is defined by the
    /// qc_type attribute.
    ///
    /// The q_code contains the actual query-code value, according to the
    /// qc_type attribute.
    ///
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
    ///
    /// The values of this attribute indicate the type if
    /// misclassification:
    ///  * posn - a mistake in the division of the kanji
    ///  * stroke_count - a mistake in the number of strokes
    ///  * stroke_and_posn - mistakes in both division and strokes
    ///  * stroke_diff - ambiguous stroke counts depending on glyph
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub query: Vec<Query>,
    /// The reading element contains the reading or pronunciation
    /// of the kanji.
    ///
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
    ///      Another attribute r_status, if present, will indicate with
    ///      a value of "jy" whether the reading is approved for a
    ///      "Jouyou kanji". (The r_status attribute is not currently used.)
    ///      A further attribute on_type, if present, will indicate with
    ///      a value of kan, go, tou or kan'you the type of on-reading.
    ///      (The on_type attribute is not currently used.)
    ///  * ja_kun - the "kun" Japanese reading of the kanji, usually in
    ///      hiragana.
    ///      Where relevant the okurigana is also included separated by a
    ///      ".". Readings associated with prefixes and suffixes are
    ///      marked with a "-". A second attribute r_status, if present,
    ///      will indicate with a value of "jy" whether the reading is
    ///      approved for a "Jouyou kanji". (The r_status attribute is
    ///      not currently used.)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub readings: Vec<Reference>,
    /// The meaning associated with the kanji.
    ///
    /// The m_lang attribute defines the target language of the meaning. It
    /// will be coded using the two-letter language code from the ISO 639-1
    /// standard. When absent, the value "en" (i.e. English) is implied. [{}]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub meanings: Vec<Reference>,
    /// Japanese readings that are now only associated with names.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nanori: Vec<String>,
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Reference<T = String> {
    pub value: T,
    #[serde(rename = "type")]
    pub typ: String,
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Query {
    pub value: String,
    #[serde(rename = "type")]
    pub typ: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub misclass: Option<String>,
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

    assert!(k.literal != char::default(), "need a literal!");
    assert!(
        k.stroke_count != i32::default(),
        "need at least one stroke count!"
    );
    return k;
}

fn parse_literal(node: Node, entry: &mut Kanji) {
    let text = node.text().expect("failed to get text").trim();
    assert!(
        text.chars().count() == 1,
        "only expected single char for literal, got {}",
        text
    );
    let c = text.chars().next().expect("no first char");
    entry.literal = c
}

fn parse_codepoint(node: Node, entry: &mut Kanji) {
    entry.codepoint = node
        .children()
        .filter(|n| n.is_element())
        .map(|n| Reference {
            value: get_text(n.text()),
            typ: get_text(n.attribute("cp_type")),
        })
        .collect();
}

fn parse_radical(node: Node, entry: &mut Kanji) {
    entry.radical = node
        .children()
        .filter(|n| n.is_element())
        .map(|n| Reference {
            value: get_num(n.text()),
            typ: get_text(n.attribute("rad_type")),
        })
        .collect();
}

fn parse_misc(node: Node, entry: &mut Kanji) {
    for n in node.children().filter(|n| n.is_element()) {
        match n.tag_name().name() {
            "grade" => entry.grade = Some(get_num(n.text())),
            "stroke_count" => {
                let count = get_num(n.text());
                if entry.stroke_count == 0 {
                    entry.stroke_count = count
                } else {
                    entry.stroke_miscounts.push(count)
                }
            }
            "variant" => entry.variant.push(Reference {
                value: get_text(n.text()),
                typ: get_text(n.attribute("var_type")),
            }),
            "freq" => entry.freq = Some(get_num(n.text())),
            "rad_name" => entry.names.push(get_text(n.text())),
            "jlpt" => entry.jlpt = Some(get_num(n.text())),
            tag => println!("Warning: unexpected tag name in misc: {}", tag),
        };
    }
}

fn parse_dic_number(node: Node, entry: &mut Kanji) {
    entry.dict = node
        .children()
        .filter(|n| n.is_element())
        .map(|n| {
            let typ = get_text(n.attribute("dr_type"));

            let text = get_text(n.text());
            let value = match text.parse::<i32>() {
                Ok(i) => NumString::Num(i),
                Err(_) => NumString::String(text),
            };

            Reference { value, typ }
        })
        .collect();
}

fn parse_query_code(node: Node, entry: &mut Kanji) {
    entry.query = node
        .children()
        .filter(|n| n.is_element())
        .map(|n| Query {
            value: get_text(n.text()),
            typ: get_text(n.attribute("qc_type")),
            misclass: n.attribute("skip_misclass").map(|s| s.trim().into()),
        })
        .collect();
}

fn parse_reading_meaning(node: Node, entry: &mut Kanji) {
    for n in node.children().filter(|n| n.is_element()) {
        assert!(
            n.children().filter(|n| n.has_tag_name("rmgroup")).count() <= 1,
            "cannot have more than one rmgroup!"
        );

        match n.tag_name().name() {
            "rmgroup" => parse_rmgroup(n, entry),
            "nanori" => entry.nanori.push(get_text(n.text())),
            tag => println!("Warning: unexpected tag name in reading_meaning: {}", tag),
        };
    }
}

fn parse_rmgroup(node: Node, entry: &mut Kanji) {
    for n in node.children().filter(|n| n.is_element()) {
        match n.tag_name().name() {
            "reading" => entry.readings.push(Reference {
                value: get_text(n.text()),
                typ: get_text(n.attribute("r_type")),
            }),
            "meaning" => entry.meanings.push(Reference {
                value: get_text(n.text()),
                typ: n
                    .attribute("m_lang")
                    .map(|s| s.trim())
                    .unwrap_or("en")
                    .into(),
            }),
            tag => println!("Warning: unexpected tag name in reading_meaning: {}", tag),
        };
    }
}

fn get_text(s: Option<&str>) -> String {
    // TODO this should probably return a result instead of expecting
    s.expect("no text").trim().into()
}

fn get_num(s: Option<&str>) -> i32 {
    // TODO this should probably return a result instead of expecting
    s.expect("no text").trim().parse().expect("failed to parse")
}

#[test]
fn test_parse() {
    let mut d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("../data/kanjidic2.xml");
    let text = std::fs::read_to_string(d).unwrap();
    for kanji in parse(&text).entries() {
        println!("{:?}", kanji);
    }
}
