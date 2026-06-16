use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScannerReport {
    pub files_scanned: usize,
    pub parsed_files: usize,
    pub modules_scanned: usize,
    pub symbols_extracted: usize,
    pub imports_found: usize,
    pub relationships_created: usize,
    pub extraction_success_rate: f64,
}
