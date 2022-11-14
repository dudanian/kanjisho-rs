use roxmltree::{Document, Node, ParsingOptions};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Header {
    file_version: i32,
    database_version: String,
    date_of_creation: String,
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Kanji {
    pub literal: char,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub codepoint: Vec<Reference>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub radical: Vec<Reference<i32>>,

    // start misc
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grade: Option<i32>,
    pub stroke_count: Vec<i32>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub variant: Vec<Reference>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freq: Option<i32>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub rad_name: Vec<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jlpt: Option<i32>,
    // end misc
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dict: Vec<Reference>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub query: Vec<Query>,
    // start reading_meaning
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    // TODO there is never an instance where this is > 1
    // so why bother making it a vec?
    pub entries: Vec<ReadingMeaning>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub nanori: Vec<String>,
    // end reading_meaning
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct ReadingMeaning {
    pub readings: Vec<Part>,
    pub meanings: Vec<Part>,
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Reference<T = String> {
    pub value: T,
    pub source: String,
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Dict {
    pub value: String,
    pub source: String,
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Query {
    pub value: String,
    pub source: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub misclass: Option<String>,
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Part {
    pub value: String,
    pub lang: String,
}

/// The base Kanjidic container
///
/// Holds a reference to the parsed xml tree so we can
/// create an iterator of the entries.
pub struct Kanjidic<'a> {
    doc: Document<'a>,
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

    return k;
}

fn parse_literal(node: Node, entry: &mut Kanji) {
    let text = node.text().expect("failed to get text").trim();
    debug_assert!(
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
            source: get_text(n.attribute("cp_type")),
        })
        .collect();
}

fn parse_radical(node: Node, entry: &mut Kanji) {
    entry.radical = node
        .children()
        .filter(|n| n.is_element())
        .map(|n| Reference {
            value: get_num(n.text()),
            source: get_text(n.attribute("rad_type")),
        })
        .collect();
}

fn parse_misc(node: Node, entry: &mut Kanji) {
    for n in node.children().filter(|n| n.is_element()) {
        match n.tag_name().name() {
            "grade" => entry.grade = Some(get_num(n.text())),
            "stroke_count" => entry.stroke_count.push(get_num(n.text())),
            "variant" => entry.variant.push(Reference {
                value: get_text(n.text()),
                source: get_text(n.attribute("var_type")),
            }),
            "freq" => entry.freq = Some(get_num(n.text())),
            "rad_name" => entry.rad_name.push(get_text(n.text())),
            "jlpt" => entry.jlpt = Some(get_num(n.text())),
            tag => println!("Warning: unexpected tag name in misc: {}", tag),
        };
    }
}

fn parse_dic_number(node: Node, entry: &mut Kanji) {
    entry.dict = node
        .children()
        .filter(|n| n.is_element())
        .map(|n| Reference {
            value: get_text(n.text()),
            source: get_text(n.attribute("dr_type")),
        })
        .collect();
}

fn parse_query_code(node: Node, entry: &mut Kanji) {
    entry.query = node
        .children()
        .filter(|n| n.is_element())
        .map(|n| Query {
            value: get_text(n.text()),
            source: get_text(n.attribute("qc_type")),
            misclass: n.attribute("skip_misclass").map(|s| s.trim().into()),
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
            "reading" => group.readings.push(Part {
                value: get_text(n.text()),
                lang: get_text(n.attribute("r_type")),
            }),
            "meaning" => group.meanings.push(Part {
                value: get_text(n.text()),
                lang: n
                    .attribute("m_lang")
                    .map(|s| s.trim())
                    .unwrap_or("en")
                    .into(),
            }),
            tag => println!("Warning: unexpected tag name in reading_meaning: {}", tag),
        };
    }

    entry.entries.push(group);
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
