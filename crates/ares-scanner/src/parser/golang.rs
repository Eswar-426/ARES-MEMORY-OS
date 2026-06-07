// Go parser stub — implemented Week 7, Day 2
use ares_core::{AresError, Language};
use crate::parser::traits::{LanguageParser, ParsedFile};
use std::path::Path;

pub struct GoParser;

impl LanguageParser for GoParser {
    fn language(&self) -> Language { Language::Go }
    fn extensions(&self) -> &[&'static str] { &["go"] }
    fn parse(&self, _source: &str, file_path: &Path) -> Result<ParsedFile, AresError> {
        Ok(ParsedFile {
            file_path: file_path.to_path_buf(),
            language: Language::Go,
            functions: vec![], classes: vec![], imports: vec![], exports: vec![],
            complexity_score: 0.0, loc: 0, parse_errors: vec![],
        })
    }
}
