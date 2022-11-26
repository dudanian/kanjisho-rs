mod db;

fn main() {
    db::mongo::update_kanjidic().expect("failed to update kanjidic");
}
