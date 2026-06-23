use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalTelemetry {
    pub timestamp: DateTime<Utc>,
    pub query: String,
    pub intent: String,
    pub retrieval_time_ms: u64,
    pub nodes_examined: usize,
    pub nodes_returned: usize,
    pub max_depth: usize,
    pub confidence_score: f32,
}

pub struct TelemetryCollector {
    storage_dir: PathBuf,
}

impl TelemetryCollector {
    pub fn new(workspace_root: &str) -> Self {
        let mut path = PathBuf::from(workspace_root);
        path.push("artifacts");
        path.push("telemetry");
        fs::create_dir_all(&path).unwrap_or_default();
        Self { storage_dir: path }
    }

    pub fn record(&self, telemetry: RetrievalTelemetry) -> Result<()> {
        let timestamp_str = telemetry.timestamp.format("%Y%m%d_%H%M%S").to_string();
        let filename = format!("telemetry_{}.json", timestamp_str);

        let mut path = self.storage_dir.clone();
        path.push(filename);

        let json = serde_json::to_string_pretty(&telemetry)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn save_json(&self, telemetry_list: &[RetrievalTelemetry], filename: &str) -> Result<()> {
        let mut path = self.storage_dir.clone();
        path.push(filename);

        let json = serde_json::to_string_pretty(telemetry_list)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load_history(&self) -> Result<Vec<RetrievalTelemetry>> {
        let mut history = Vec::new();
        if !self.storage_dir.exists() {
            return Ok(history);
        }

        for entry in fs::read_dir(&self.storage_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(&path)?;
                if let Ok(telemetry) = serde_json::from_str::<RetrievalTelemetry>(&content) {
                    history.push(telemetry);
                } else if let Ok(telemetry_list) =
                    serde_json::from_str::<Vec<RetrievalTelemetry>>(&content)
                {
                    history.extend(telemetry_list);
                }
            }
        }

        history.sort_by_key(|a| a.timestamp);
        Ok(history)
    }
}
