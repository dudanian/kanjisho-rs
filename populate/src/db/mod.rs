pub mod mongo;
pub mod redis;

fn read_dict() -> String {
    let mut d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("../data/kanjidic2.xml");
    std::fs::read_to_string(d).unwrap()
}
