use rusqlite::Connection;
use std::fs::File;
use std::io::Write;

fn main() {
    let conn = Connection::open("ares_memory.db").unwrap();
    let mut stmt = conn.prepare("SELECT label, node_type, file_path FROM graph_nodes").unwrap();
    let rows = stmt.query_map([], |row| {
        let label: String = row.get(0)?;
        let node_type: String = row.get(1)?;
        let file_path: Option<String> = row.get(2)?;
        Ok(format!("{}: {} in {:?}", label, node_type, file_path))
    }).unwrap();

    let mut file = File::create("components_clean.txt").unwrap();
    for row in rows {
        let r = row.unwrap();
        writeln!(file, "{}", r).unwrap();
    }
}
