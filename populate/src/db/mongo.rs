use mongodb::sync::{Client, Collection};
use parse::kanjidic::{self, Kanji, Reference};

use super::{read_dict, read_klc_map};

fn connect() -> mongodb::error::Result<Collection<Kanji>> {
    let url = std::env::var("MONGODB_URL").expect("MONGODB_URL not set");
    let client = Client::with_uri_str(url)?;

    let database = client.database("kanjisho");
    Ok(database.collection::<Kanji>("kanjidic"))
}

pub fn update_kanjidic() -> mongodb::error::Result<()> {
    let con = connect()?;
    let text = read_dict();
    let klc = read_klc_map();

    con.insert_many(
        kanjidic::parse(&text).entries().map(|mut e| {
            if let Some(v) = klc.get(&e.literal) {
                e.dict.push(Reference {
                    value: kanjidic::NumString::Num(*v),
                    typ: "klc".into(),
                })
            }
            e
        }),
        None,
    )?;

    Ok(())
}
