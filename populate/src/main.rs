extern crate redis;
use redis::{Commands, ConnectionLike};
use serde::{Deserialize, Serialize};
use std::env;

use redis::JsonCommands;

use parse::kanjidic::parse;

#[derive(Serialize, Deserialize, Debug)]
struct Test {
    name: String,
    value: u32,
}

fn fetch_an_integer() -> redis::RedisResult<isize> {
    // connect to redis
    let url = env::var("REDIS_URL").expect("no url");
    let client = redis::Client::open(url)?;
    let mut con = client.get_connection()?;
    // throw away the result, just make sure it does not fail
    let _: () = con.set("my_key", 42)?;

    let v = Test {
        name: String::from("hello"),
        value: 32,
    };

    con.json_set("obj", "$", &v)?;

    // println!("database {}", con.get_db());

    // redis::pipe()
    //     .atomic()
    //     .cmd("SELECT")
    //     .arg(1)
    //     .ignore()
    //     .query(&mut con)?;

    // println!("database {}", con.get_db());

    let v2: String = con.json_get("obj", "$")?;

    println!("value from redis {}", v2);

    let v3: Vec<Test> = serde_json::from_str(v2.as_str())?;

    println!("value from serde {:?}", v3);

    let _: () = con.set("our_key", 43)?;

    // read back the key and return it.  Because the return value
    // from the function is a result for integer this will automatically
    // convert into one.
    con.get("our_key")
}

fn do_it() -> redis::RedisResult<()> {
    let url = env::var("REDIS_URL").expect("no url");
    let client = redis::Client::open(url)?;
    let mut con = client.get_connection()?;
    // throw away the result, just make sure it does not fail

    let mut d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("../data/kanjidic2.xml");
    let text = std::fs::read_to_string(d).unwrap();
    let mut v = Vec::new();
    for kanji in parse(&text).entries() {
        let s = serde_json::to_string(&kanji).expect("failed to serialize");
        let c = kanji.literal.to_string();

        // con.set(&c, s)?;

        v.push(kanji.literal);
    }
    let i = serde_json::to_string(&v)?;
    con.set("index", i)?;

    Ok(())
}

fn main() {
    //let i = fetch_an_integer().expect("failed");

    // println!("number {}", i);

    do_it().expect("failed to do it");
}
