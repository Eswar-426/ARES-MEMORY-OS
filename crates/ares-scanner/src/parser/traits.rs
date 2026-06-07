use ares_core::{AresError, Language};
use std::path::Path;

/// A parsed representation of a source file.
#[derive(Debug, Clone)]
pub struct ParsedFile {
    pub file_path: std::path::PathBuf,
    pub language: Language,
    pub functions: Vec<ExtractedFunction>,
    pub classes: Vec<ExtractedClass>,
    pub imports: Vec<ExtractedImport>,
    pub exports: Vec<ExtractedExport>,
    pub complexity_score: f32,
    pub loc: u32,
    pub parse_errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ExtractedFunction {
    pub name: String,
    pub start_line: u32,
    pub end_line: u32,
    pub is_async: bool,
    pub is_exported: bool,
}

#[derive(Debug, Clone)]
pub struct ExtractedClass {
    pub name: String,
    pub start_line: u32,
    pub end_line: u32,
    pub superclass: Option<String>,
    pub is_exported: bool,
}

#[derive(Debug, Clone)]
pub struct ExtractedImport {
    pub module_path: String,
    pub names: Vec<String>,
    pub is_default: bool,
}

#[derive(Debug, Clone)]
pub struct ExtractedExport {
    pub name: String,
    pub export_type: String,
}

/// Core trait all language parsers must implement.
pub trait LanguageParser: Send + Sync {
    fn language(&self) -> Language;
    fn extensions(&self) -> &[&'static str];
    fn parse(&self, source: &str, file_path: &Path) -> Result<ParsedFile, AresError>;
    fn can_parse(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|ext| self.extensions().contains(&ext))
            .unwrap_or(false)
    }
}
