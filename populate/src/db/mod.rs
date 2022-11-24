use std::collections::HashMap;

pub mod mongo;
pub mod redis;

fn read_dict() -> String {
    let mut d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("../data/kanjidic2.xml");
    std::fs::read_to_string(d).unwrap()
}

fn read_klc_map() -> HashMap<char, i32> {
    let mut d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("../data/klc.txt");

    let mut m = HashMap::new();

    let s = std::fs::read_to_string(d).unwrap();
    let mut i = 1;

    for l in s.lines() {
        for c in l.chars() {
            m.insert(c, i);
            i += 1;
        }
    }

    m
}
