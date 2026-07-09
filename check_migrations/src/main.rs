use refinery::embed_migrations;
embed_migrations!("../crates/ares-store/src/migrations");
fn main() {
    let r = migrations::runner();
    for m in r.get_migrations() {
        if m.version() == 6 {
            println!("V6 checksum: {}", m.checksum());
        }
    }
}
