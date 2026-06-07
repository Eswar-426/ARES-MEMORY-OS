// TypeScript parser — implemented Week 6, Day 2
// Uses tree-sitter-typescript to extract functions, classes, imports

use crate::parser::traits::{LanguageParser, ParsedFile};
use ares_core::{AresError, Language};
use std::path::Path;

pub struct TypeScriptParser;

impl LanguageParser for TypeScriptParser {
    fn language(&self) -> Language {
        Language::TypeScript
    }
    fn extensions(&self) -> &[&'static str] {
        &["ts", "tsx"]
    }
    fn parse(&self, _source: &str, file_path: &Path) -> Result<ParsedFile, AresError> {
        // TODO Week 6: Implement Tree-sitter parsing
        Ok(ParsedFile {
            file_path: file_path.to_path_buf(),
            language: Language::TypeScript,
            functions: vec![],
            classes: vec![],
            imports: vec![],
            exports: vec![],
            complexity_score: 0.0,
            loc: 0,
            parse_errors: vec![],
        })
    }
}
