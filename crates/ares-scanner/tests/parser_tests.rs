use ares_scanner::languages::{rust::RustExtractor, LanguageExtractor};
use ares_core::ProjectId;

#[test]
fn test_rust_extractor() {
    let extractor = RustExtractor::new();
    let code = r#"
        pub struct MyStruct {
            field: i32,
        }
        impl MyStruct {
            fn my_method() {}
        }
        fn main() {}
    "#;
    let project_id = ProjectId::new();
    let result = extractor.extract(&project_id, "main.rs", code).unwrap();
    
    // We should find MyStruct (Struct), my_method (Method/Function double capture), main (Function)
    assert_eq!(result.nodes.len(), 4);
    
    let labels: Vec<String> = result.nodes.iter().map(|n| n.label.clone()).collect();
    assert!(labels.contains(&"MyStruct".to_string()));
    assert!(labels.contains(&"my_method".to_string()));
    assert!(labels.contains(&"main".to_string()));
}
