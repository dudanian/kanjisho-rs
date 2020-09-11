mod de;

use de::option_as_bool;
use serde::Deserialize;
use serde_webdoc::{into_iter, ElementIter};
use web_sys::Element;

impl Entry {
    pub fn elem_iter(elem: Element) -> ElementIter<Self> {
        into_iter(elem.first_element_child())
    }
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Entry {
    /// unique numeric sequence number
    #[serde(rename = "ent_seq")]
    pub sequence: i32,
    /// the kanji elements
    #[serde(rename = "k_ele", default)]
    pub kanjis: Option<Vec<Kanji>>,
    /// the reading elements
    #[serde(rename = "r_ele", default)]
    pub readings: Vec<Reading>,
    /// the meaning elements
    #[serde(rename = "sense", default)]
    pub senses: Vec<Sense>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Kanji {
    /// a word or short phrase in Japanese
    #[serde(rename = "keb")]
    pub element: String,
    /// orthography information
    #[serde(rename = "ke_inf", default)]
    pub information: Option<Vec<String>>,
    /// relative priorities
    #[serde(rename = "ke_pri", default)]
    pub priorities: Option<Vec<String>>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Reading {
    /// kana reading for kanji entry
    #[serde(rename = "reb")]
    pub element: String,
    /// cannot be regarded as a true reading
    #[serde(rename = "re_nokanji", default, deserialize_with = "option_as_bool")]
    pub nokanji: bool,
    // TODO use deserialize_with to fix weird bool thing
    /// reading only applies to this subset of kanji elements
    #[serde(rename = "re_restr", default)]
    pub restricted_kanjis: Option<Vec<String>>,
    /// additional information
    #[serde(rename = "re_inf", default)]
    pub information: Option<Vec<String>>,
    /// relative priorities
    #[serde(rename = "re_pri", default)]
    pub priorities: Option<Vec<String>>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Sense {
    // TODO additional gloss info
    /// words or phrases which are equivalent to Japanese entry
    #[serde(rename = "gloss", default)]
    pub meanings: Option<Vec<String>>,
    /// sense only applies to this subset of kanji elements
    #[serde(rename = "stagk", default)]
    pub restricted_kanjis: Option<Vec<String>>,
    /// sense only applies to this subset of reading elements
    #[serde(rename = "stagr", default)]
    pub restricted_readings: Option<Vec<String>>,
    /// additional information
    #[serde(rename = "s_inf", default)]
    pub information: Option<Vec<String>>,

    // TODO additional lsource info
    /// source languages of loan words
    #[serde(rename = "lsource", default)]
    pub source_languages: Option<Vec<String>>,
    /// regional dialects
    #[serde(rename = "dial", default)]
    pub dialects: Option<Vec<String>>,
    /// parts of speech
    #[serde(rename = "pos", default)]
    pub parts_of_speech: Option<Vec<String>>,
    /// cross references
    #[serde(rename = "xref", default)]
    pub cross_references: Option<Vec<String>>,
    /// antonyms
    #[serde(rename = "ant", default)]
    pub antonyms: Option<Vec<String>>,
    /// fields of application
    #[serde(rename = "field", default)]
    pub fields: Option<Vec<String>>,
    /// miscellaneous information
    #[serde(rename = "misc", default)]
    pub misc: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
    use web_sys::{DomParser, SupportedType};

    use serde_webdoc::from_doc;
    use serde_webdoc::into_iter;
    use serde_webdoc::Result;

    wasm_bindgen_test_configure!(run_in_browser);

    macro_rules! parse_from_string {
        ($text:ident) => {{
            let parser = DomParser::new().unwrap();
            parser
                .parse_from_string(&$text, SupportedType::TextXml)
                .unwrap()
        }};
    }

    #[wasm_bindgen_test]
    fn parse_entry() {
        let text = "
<entry>
<ent_seq>1000220</ent_seq>
<k_ele>
<keb>明白</keb>
<ke_pri>ichi1</ke_pri>
<ke_pri>news1</ke_pri>
<ke_pri>nf10</ke_pri>
</k_ele>
<r_ele>
<reb>めいはく</reb>
<re_nokanji></re_nokanji>
<re_pri>ichi1</re_pri>
<re_pri>news1</re_pri>
<re_pri>nf10</re_pri>
</r_ele>
<sense>
<gloss>obvious</gloss>
<gloss>clear</gloss>
<gloss>plain</gloss>
<gloss>evident</gloss>
<gloss>apparent</gloss>
<gloss>explicit</gloss>
<gloss>overt</gloss>
</sense>
</entry>";

        let doc = parse_from_string!(text);
        let entry: Result<Entry> = from_doc(&doc);
        assert_eq!(
            entry,
            Ok(Entry {
                sequence: 1000220,
                kanjis: Some(vec![Kanji {
                    element: "明白".into(),
                    information: None,
                    priorities: Some(vec!["ichi1".into(), "news1".into(), "nf10".into()])
                }]),
                readings: vec![Reading {
                    element: "めいはく".into(),
                    nokanji: true,
                    restricted_kanjis: None,
                    information: None,
                    priorities: Some(vec!["ichi1".into(), "news1".into(), "nf10".into()])
                }],
                senses: vec![Sense {
                    meanings: Some(vec![
                        "obvious".into(),
                        "clear".into(),
                        "plain".into(),
                        "evident".into(),
                        "apparent".into(),
                        "explicit".into(),
                        "overt".into()
                    ]),
                    restricted_kanjis: None,
                    restricted_readings: None,
                    information: None,
                    source_languages: None,
                    dialects: None,
                    parts_of_speech: None,
                    cross_references: None,
                    antonyms: None,
                    fields: None,
                    misc: None
                }]
            })
        );
    }

    #[wasm_bindgen_test]
    fn parse_entry_iter() {
        let text = "<dict>
<entry>
<ent_seq>1</ent_seq>
<r_ele>
<reb>one</reb>
</r_ele>
<sense>
</sense>
</entry>
<entry>
<ent_seq>2</ent_seq>
<r_ele>
<reb>two</reb>
</r_ele>
<sense>
</sense>
</entry>
<entry>
<ent_seq>3</ent_seq>
<r_ele>
<reb>three</reb>
</r_ele>
<sense>
</sense>
</entry>
</dict>";

        let doc = parse_from_string!(text);
        let root = doc.first_element_child().unwrap();
        let mut iter = Entry::elem_iter(root);
        assert_eq!(
            iter.next(),
            Some(Ok(Entry {
                sequence: 1,
                kanjis: None,
                readings: vec![Reading {
                    element: "one".into(),
                    nokanji: false,
                    restricted_kanjis: None,
                    information: None,
                    priorities: None
                }],
                senses: vec![Sense {
                    meanings: None,
                    restricted_kanjis: None,
                    restricted_readings: None,
                    information: None,
                    source_languages: None,
                    dialects: None,
                    parts_of_speech: None,
                    cross_references: None,
                    antonyms: None,
                    fields: None,
                    misc: None
                }]
            }))
        );
        assert_eq!(
            iter.next(),
            Some(Ok(Entry {
                sequence: 2,
                kanjis: None,
                readings: vec![Reading {
                    element: "two".into(),
                    nokanji: false,
                    restricted_kanjis: None,
                    information: None,
                    priorities: None
                }],
                senses: vec![Sense {
                    meanings: None,
                    restricted_kanjis: None,
                    restricted_readings: None,
                    information: None,
                    source_languages: None,
                    dialects: None,
                    parts_of_speech: None,
                    cross_references: None,
                    antonyms: None,
                    fields: None,
                    misc: None
                }]
            }))
        );
        assert_eq!(
            iter.next(),
            Some(Ok(Entry {
                sequence: 3,
                kanjis: None,
                readings: vec![Reading {
                    element: "three".into(),
                    nokanji: false,
                    restricted_kanjis: None,
                    information: None,
                    priorities: None
                }],
                senses: vec![Sense {
                    meanings: None,
                    restricted_kanjis: None,
                    restricted_readings: None,
                    information: None,
                    source_languages: None,
                    dialects: None,
                    parts_of_speech: None,
                    cross_references: None,
                    antonyms: None,
                    fields: None,
                    misc: None
                }]
            }))
        );
        assert_eq!(iter.next(), None);
    }
}
