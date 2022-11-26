// I should probably start defining a different struct here than for parsing
// since those are two different things

// for example, it would be good to have `meta` or something for db stuff
// like sorting, grouping, etc that doesn't really apply to the char itself

use serde::{Deserialize, Serialize};

struct Meta {
    skip_1: i32,
    skip_2: i32,
    skip_3: i32,
}

#[derive(Default, Serialize, Deserialize, Debug, PartialEq)]
struct Codepoint {
    /// Unicode 4.0 - hex coding (4 or 5 hexadecimal digits)
    ucs: String,
    /// JIS X 0208-1997 - kuten coding (nn-nn)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    jis208: Option<String>,
    /// JIS X 0212-1990 - kuten coding (nn-nn)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    jis212: Option<String>,
    /// JIS X 0213-2000 - kuten coding (p-nn-nn)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    jis213: Option<String>,
}

struct Radical {
    /// The radical number, in the range 1 to 214.
    /// based on the system first used in the KangXi Zidian.
    classic: i32,
    /// The radical number, in the range 1 to 214.
    /// as used in the Nelson "Modern Japanese-English Character Dictionary"
    nelson: i32,
    /// The radical number (if it is a radical)
    number: Option<i32>,
    /// List of radical names (if it is a radical)
    name: Vec<String>,
}

struct Info {
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

    pub jlptn: Option<i32>,
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
struct Kanji {
    /// The character itself in UTF8 coding.
    pub literal: char,

    #[serde(default)]
    pub codepoint: Codepoint,

    pub radical: Radical,

    pub info: Info,

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

    pub jlptn: Option<i32>,

    /// The "on" Japanese reading of the kanji, in katakana.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub on_readings: Vec<String>,
    /// The "kun" Japanese reading of the kanji, usually in hiragana.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub kun_readings: Vec<String>,
    /// The meaning associated with the kanji. (in English)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub meanings: Vec<String>,
    /// Japanese readings that are now only associated with names.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nanori: Vec<String>,
}
