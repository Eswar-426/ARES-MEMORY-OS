use ares_context::query::parser::QueryParser;
fn main() {
    let p = QueryParser::new();
    let targets = p.extract_targets("Trace scanner dependencies Trace all internal and external dependencies utilized by the ares-scanner crate for analyzing the AST.");
    println!("Targets: {:?}", targets);
}
