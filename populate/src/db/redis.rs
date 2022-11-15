extern crate redis;

use redis::Commands;
use std::env;

use parse::kanjidic;

use super::read_dict;

fn connect() -> Result<redis::Connection, redis::RedisError> {
    let url = env::var("REDIS_URL").expect("REDIS_URL not set");
    let client = redis::Client::open(url)?;
    client.get_connection()
}

pub fn update_kanjidic() -> redis::RedisResult<()> {
    let mut con = connect()?;
    let text = read_dict();
    let mut index = Vec::new();

    for kanji in kanjidic::parse(&text).entries() {
        let key = kanji.literal.to_string();
        let value = serde_json::to_string(&kanji).expect("failed to serialize kanji");

        con.set(&key, value)?;
        index.push(kanji.literal);
    }
    let value = serde_json::to_string(&index).expect("failed to serialize index");
    con.set("index", value)?;

    Ok(())
}
