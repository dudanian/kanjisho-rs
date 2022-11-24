mod db;

fn main() {
    // db::redis::update_kanjidic().expect("failed to update kanjidic");
    db::mongo::update_kanjidic().expect("failed to update kanjidic");
}
