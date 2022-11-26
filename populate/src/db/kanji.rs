use std::collections::HashMap;

use backend::data::kanji;
use parse::kanjidic;

#[derive(Debug)]
pub enum Error {
    NoLiteral,
    NoStrokeCount(char),
    NoRadical(char),
    BadRadical(char),
    NoUcs(char),
}

/// Convert a Kanjidic entry into a backend Kanji entry
/// Check for anything I might want guaranteed, like potentially missing
/// elements and add missing information from other sources.
pub fn convert(
    k: &kanjidic::Kanji,
    jlpt: &HashMap<char, u32>,
    klc: &HashMap<char, u32>,
) -> Result<kanji::Kanji, Error> {
    if k.literal == char::default() {
        return Err(Error::NoLiteral);
    }

    let rmgroup = k.rmgroup.first();

    let classic = k
        .radical
        .iter()
        .find(|r| r.rad_type == "classical")
        .ok_or(Error::NoRadical(k.literal))?
        .rad_value;
    let nelson = k
        .radical
        .iter()
        .find(|r| r.rad_type == "nelson_c")
        .map(|r| r.rad_value);

    let stroke_count = k
        .stroke_count
        .first()
        .ok_or(Error::NoStrokeCount(k.literal))?
        .to_owned();

    let ucs = k
        .codepoint
        .iter()
        .find(|c| c.cp_type == "ucs")
        .ok_or(Error::NoUcs(k.literal))?
        .cp_value
        .clone();

    let rtk = k
        .dic_number
        .iter()
        .find(|d| d.dr_type == "heisig6")
        .map(|d| d.dic_ref.parse::<u32>())
        .map_or(Ok(None), |r| r.map(Some))
        .map_err(|_e| Error::NoLiteral)?;

    Ok(kanji::Kanji {
        literal: k.literal,
        info: kanji::Info {
            radical: classic,
            radical_n: nelson.unwrap_or(classic),
            stroke_count,
            grade: k.grade,
            freq: k.freq,
            jlpt: k.jlpt,
            jlptn: jlpt.get(&k.literal).copied(),
        },
        references: kanji::References {
            ucs,
            jis208: None,
            jis212: None,
            jis213: None,
            rtk,
            klc: klc.get(&k.literal).copied(),
        },
        on_readings: rmgroup
            .map(|g| {
                g.reading
                    .iter()
                    .filter(|r| r.r_type == "ja_on")
                    .map(|r| r.reading.clone())
                    .collect()
            })
            .unwrap_or_default(),
        kun_readings: rmgroup
            .map(|g| {
                g.reading
                    .iter()
                    .filter(|r| r.r_type == "ja_kun")
                    .map(|r| r.reading.clone())
                    .collect()
            })
            .unwrap_or_default(),
        meanings: rmgroup
            .map(|g| {
                g.meaning
                    .iter()
                    .filter(|m| m.m_lang == "en")
                    .map(|m| m.meaning.clone())
                    .collect()
            })
            .unwrap_or_default(),
        nanoris: k.nanori.clone(),
    })
}
