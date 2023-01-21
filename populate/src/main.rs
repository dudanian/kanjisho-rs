mod db;

fn main() {
    // db::mongo::update_kanjidic().expect("failed to update kanjidic");
    db::json::update_kanjidic()
}
