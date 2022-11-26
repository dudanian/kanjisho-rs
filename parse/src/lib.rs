pub mod kanjidic;
use std::collections::HashMap;

fn data_path(file: &str) -> std::path::PathBuf {
    [env!("CARGO_MANIFEST_DIR"), "../data", file]
        .iter()
        .collect()
}

pub fn read_kanjidic() -> String {
    std::fs::read_to_string(data_path("kanjidic2.xml")).unwrap()
}

pub fn read_klc() -> String {
    std::fs::read_to_string(data_path("klc.txt")).unwrap()
}

/// Maps a list of input charactecrs to their repsective 1-offset index.
/// Any duplicate charactercs will result in that character being returned
/// as an error.
///
/// While parsing the list:
///  - newlines are ignored
///  - space characters can be used to skip entries
///  - any other unicode character will be mapped
pub fn read_char_index(text: &str) -> Result<HashMap<char, u32>, char> {
    let mut m = HashMap::new();
    let mut i = 0;

    for l in text.lines() {
        for c in l.chars() {
            i += 1;
            if c == ' ' {
                continue;
            }
            if let Some(_) = m.insert(c, i) {
                return Err(c);
            }
        }
    }

    Ok(m)
}
