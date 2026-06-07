use crate::languages::{
    go::GoExtractor, javascript::JavaScriptExtractor, python::PythonExtractor, rust::RustExtractor,
    typescript::TypeScriptExtractor, ExtractionResult, LanguageExtractor,
};
use ares_core::ProjectId;

pub struct ExtractorRouter {
    rust: RustExtractor,
    ts: TypeScriptExtractor,
    js: JavaScriptExtractor,
    python: PythonExtractor,
    go: GoExtractor,
}

impl ExtractorRouter {
    pub fn new() -> Self {
        Self {
            rust: RustExtractor::new(),
            ts: TypeScriptExtractor::new(),
            js: JavaScriptExtractor::new(),
            python: PythonExtractor::new(),
            go: GoExtractor::new(),
        }
    }

    pub fn extract(
        &self,
        project_id: &ProjectId,
        file_path: &str,
        source_code: &str,
    ) -> Result<Option<ExtractionResult>, Box<dyn std::error::Error + Send + Sync>> {
        let ext = std::path::Path::new(file_path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        let extractor: &dyn LanguageExtractor = match ext {
            "rs" => &self.rust,
            "ts" | "tsx" => &self.ts,
            "js" | "jsx" => &self.js,
            "py" => &self.python,
            "go" => &self.go,
            _ => return Ok(None),
        };

        Ok(Some(extractor.extract(
            project_id,
            file_path,
            source_code,
        )?))
    }
}

impl Default for ExtractorRouter {
    fn default() -> Self {
        Self::new()
    }
}
