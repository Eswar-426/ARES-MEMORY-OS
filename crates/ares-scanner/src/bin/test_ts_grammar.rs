use tree_sitter::Query;

fn main() {
    let ts_lang = tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into();
    let query_str = r#"
        (function_declaration name: (identifier) @name) @function
        (method_definition name: (property_identifier) @name) @method
        (class_declaration name: (type_identifier) @name) @class
        (interface_declaration name: (type_identifier) @name) @interface
        (enum_declaration name: (identifier) @name) @enum
    "#;
    let _q = Query::new(&ts_lang, query_str).unwrap();
    println!("Compiled class_declaration!");
}
