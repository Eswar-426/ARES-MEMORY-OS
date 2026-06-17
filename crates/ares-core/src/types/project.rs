use crate::id::ProjectId;
use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────
// Project types
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum ProjectMaturity {
    #[default]
    Greenfield,
    Growth,
    Mature,
    Legacy,
}

impl ProjectMaturity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Greenfield => "greenfield",
            Self::Growth => "growth",
            Self::Mature => "mature",
            Self::Legacy => "legacy",
        }
    }
}

impl std::str::FromStr for ProjectMaturity {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "greenfield" => Ok(Self::Greenfield),
            "growth" => Ok(Self::Growth),
            "mature" => Ok(Self::Mature),
            "legacy" => Ok(Self::Legacy),
            other => Err(format!("Unknown project maturity: {other}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    TypeScript,
    JavaScript,
    Python,
    Go,
    Java,
    CSharp,
    Rust,
    Php,
    Ruby,
    Other(String),
}

impl Language {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "ts" | "tsx" => Some(Self::TypeScript),
            "js" | "jsx" | "mjs" | "cjs" => Some(Self::JavaScript),
            "py" | "pyw" => Some(Self::Python),
            "go" => Some(Self::Go),
            "java" => Some(Self::Java),
            "cs" => Some(Self::CSharp),
            "rs" => Some(Self::Rust),
            "php" => Some(Self::Php),
            "rb" => Some(Self::Ruby),
            _ => None,
        }
    }

    pub fn is_supported(&self) -> bool {
        matches!(
            self,
            Self::TypeScript | Self::JavaScript | Self::Python | Self::Go
        )
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::TypeScript => "typescript",
            Self::JavaScript => "javascript",
            Self::Python => "python",
            Self::Go => "go",
            Self::Java => "java",
            Self::CSharp => "csharp",
            Self::Rust => "rust",
            Self::Php => "php",
            Self::Ruby => "ruby",
            Self::Other(s) => s.as_str(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    pub description: String,
    pub root_path: String,
    pub primary_language: String,
    pub domain: String,
    pub maturity: ProjectMaturity,
    pub created_at: i64,
    pub updated_at: i64,
    pub deleted_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStatus {
    pub project: Project,
    pub memory_counts: std::collections::HashMap<String, u64>,
    pub total_nodes: u64,
    pub total_edges: u64,
    pub last_scan_at: Option<i64>,
    pub last_scan_status: Option<String>,
    pub stale_decisions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectFingerprint {
    pub total_files: usize,
    pub languages: Vec<String>,
    pub crates: usize,
    pub modules: usize,
    pub hash: String,
}
