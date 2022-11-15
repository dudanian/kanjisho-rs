use mongodb::sync::{Client, Collection};
use parse::kanjidic::{self, Kanji};

use super::read_dict;

fn connect() -> mongodb::error::Result<Collection<Kanji>> {
    let url = std::env::var("MONGODB_URL").expect("MONGODB_URL not set");
    let client = Client::with_uri_str(url)?;

    let database = client.database("kanjisho");
    Ok(database.collection::<Kanji>("kanjidic"))
}

pub fn update_kanjidic() -> mongodb::error::Result<()> {
    let con = connect()?;
    let text = read_dict();

    con.insert_many(kanjidic::parse(&text).entries(), None)?;

    Ok(())
}
