use crate::languages::{
    cpp::CppExtractor, csharp::CSharpExtractor, go::GoExtractor, java::JavaExtractor,
    javascript::JavaScriptExtractor, python::PythonExtractor, ruby::RubyExtractor,
    rust::RustExtractor, typescript::TypeScriptExtractor, ExtractionResult, LanguageExtractor,
};
use ares_core::ProjectId;

pub struct ExtractorRouter {
    rust: RustExtractor,
    ts: TypeScriptExtractor,
    js: JavaScriptExtractor,
    python: PythonExtractor,
    go: GoExtractor,
    java: JavaExtractor,
    csharp: CSharpExtractor,
    cpp: CppExtractor,
    ruby: RubyExtractor,
}

impl ExtractorRouter {
    pub fn new() -> Self {
        Self {
            rust: RustExtractor::new(),
            ts: TypeScriptExtractor::new(),
            js: JavaScriptExtractor::new(),
            python: PythonExtractor::new(),
            go: GoExtractor::new(),
            java: JavaExtractor::new(),
            csharp: CSharpExtractor::new(),
            cpp: CppExtractor::new(),
            ruby: RubyExtractor::new(),
        }
    }

    pub fn extract(
        &self,
        project_id: &ProjectId,
        file_node_id: &ares_core::NodeId,
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
            "java" => &self.java,
            "cs" => &self.csharp,
            "c" | "cpp" | "h" | "hpp" => &self.cpp,
            "rb" => &self.ruby,
            _ => return Ok(None),
        };

        Ok(Some(extractor.extract(
            project_id,
            file_node_id,
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
