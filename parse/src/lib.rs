pub mod jmdict;
pub mod kanjidic;

fn data_path(file: &str) -> std::path::PathBuf {
    [env!("CARGO_MANIFEST_DIR"), "../data", file]
        .iter()
        .collect()
}

pub fn read_file(filename: &str) -> String {
    std::fs::read_to_string(data_path(filename)).unwrap()
}

pub fn write_file(filename: &str, data: &[u8]) {
    std::fs::write(data_path(filename), data).unwrap()
}

pub mod util {
    use std::collections::HashMap;

    /// Maps a list of input characters to their repsective 1-offset indexes.
    /// Any duplicate characters will result in that character being returned
    /// as an error.
    ///
    /// While parsing the list:
    ///  - newlines are ignored
    ///  - space characters can be used to skip entries
    ///  - any other unicode character will be mapped
    pub fn index_mapping(list: &str) -> Result<HashMap<char, u32>, char> {
        let mut m = HashMap::new();

        for (i, c) in char_iter(list).enumerate() {
            let index = (i + 1) as u32;
            if c == ' ' {
                continue;
            }

            if let Some(_) = m.insert(c, index) {
                return Err(c);
            }
        }

        Ok(m)
    }

    /// Maps an array of lists of characters to their 1-offset index in the input array.
    /// Any duplicate characters will result in that character being returned
    /// as an error.
    ///
    /// While parsing the list:
    ///  - newlines are ignored
    ///  - space characters can be used to skip entries
    ///  - any other unicode character will be mapped
    pub fn grade_mapping(grades: &[&str]) -> Result<HashMap<char, u32>, char> {
        let mut m = HashMap::default();

        for (i, list) in grades.iter().enumerate() {
            let grade = (i + 1) as u32;
            for c in char_iter(list) {
                if c == ' ' {
                    continue;
                }

                if let Some(_) = m.insert(c, grade) {
                    return Err(c);
                }
            }
        }

        Ok(m)
    }

    fn char_iter<'a>(list: &'a str) -> impl Iterator<Item = char> + 'a {
        list.lines().flat_map(|l| l.chars())
    }
}
