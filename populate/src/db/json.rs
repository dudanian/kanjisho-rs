use core::panic;

use backend::data::kanji::Kanji;

use parse::util;

use super::kanji::convert;

pub fn update_kanjidic() -> () {
    let klc = util::index_mapping(&parse::read_file("klc.txt")).expect("something");

    let jlpt = util::grade_mapping(&[
        &parse::read_file("n1.txt"),
        &parse::read_file("n2.txt"),
        &parse::read_file("n3.txt"),
        &parse::read_file("n4.txt"),
        &parse::read_file("n5.txt"),
    ])
    .expect("grade mapping");

    let text = parse::read_file("kanjidic2.xml");

    let entries: Vec<Kanji> = parse::kanjidic::parse(&text)
        .entries()
        .map(|k| convert(&k, &jlpt, &klc).unwrap())
        .collect();

    parse::write_file("kanjidic.json", serde_json::to_string(&entries).unwrap().as_bytes());
    
}
