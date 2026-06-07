// Python parser stub — implemented Week 7, Day 1
use crate::parser::traits::{LanguageParser, ParsedFile};
use ares_core::{AresError, Language};
use std::path::Path;

pub struct PythonParser;

impl LanguageParser for PythonParser {
    fn language(&self) -> Language {
        Language::Python
    }
    fn extensions(&self) -> &[&'static str] {
        &["py", "pyw"]
    }
    fn parse(&self, _source: &str, file_path: &Path) -> Result<ParsedFile, AresError> {
        Ok(ParsedFile {
            file_path: file_path.to_path_buf(),
            language: Language::Python,
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
