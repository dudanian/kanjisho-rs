use core::panic;

use backend::data::kanji::Kanji;
use mongodb::{
    bson::doc,
    sync::{Client, Collection},
    IndexModel,
};
use parse::util;

use super::kanji::convert;

fn connect() -> mongodb::error::Result<Collection<Kanji>> {
    let url = std::env::var("MONGODB_URL").expect("MONGODB_URL not set");
    let client = Client::with_uri_str(url)?;

    let database = client.database("kanjisho");
    Ok(database.collection::<Kanji>("kanjidic"))
}

pub fn update_kanjidic() -> mongodb::error::Result<()> {
    let con = connect()?;
    // hard reset
    con.drop(None)?;

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

    for k in parse::kanjidic::parse(&text)
        .entries()
        .map(|k| convert(&k, &jlpt, &klc))
    {
        match k {
            Ok(k) => con.insert_one(k, None)?,
            Err(e) => panic!("{:?}", e),
        };
    }

    let m = IndexModel::builder()
        .keys(doc! {
            "meanings": "text"
        })
        .build();
    // TODO I should create indexes
    con.create_index(m, None)?;

    let m = IndexModel::builder()
        .keys(doc! {
            "literal": 1
        })
        .build();
    // TODO I should create indexes
    con.create_index(m, None)?;

    let m = IndexModel::builder()
        .keys(doc! {
            "references": 1
        })
        .build();
    // TODO I should create indexes
    con.create_index(m, None)?;

    Ok(())
}
