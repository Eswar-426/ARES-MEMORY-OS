fn main() {
    let p = "E:\\My Projects\\ARES_Memory_os\\datasets\\repositories\\tokio\\tokio\\tests\\tracing-instrumentation\\Cargo.toml";
    let c = ares_core::id::canonicalize_node_id(p);
    println!("Canonicalized: {}", c);
}
