use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════
// EngineeringInsight — the universal intelligence response
// ═══════════════════════════════════════════════════════════════════

/// How the insight was produced.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InferenceMode {
    /// Purely deterministic, no AI involved.
    Offline,
    /// Deterministic + AI polish pass.
    Polished,
    /// Future: specific AI providers.
    Ollama,
    Nvidia,
    Grok,
    OpenAI,
    Claude,
}

/// A single piece of flattened evidence for UI rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    /// Category tag (e.g. "dependent", "git_commit", "owner", "requirement")
    pub category: String,
    /// Human-readable value
    pub value: String,
}

/// Confidence score with explanations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceResult {
    /// 0–100 confidence score
    pub score: u8,
    /// Human-readable reasons contributing to the score
    pub reasons: Vec<String>,
}

/// Execution metadata for observability.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InsightMetadata {
    pub duration_ms: u64,
    pub evidence_sources: Vec<String>,
    pub generator: String,
}

/// The standardized response for ALL intelligence features.
///
/// Dashboard, Graph Explorer, CLI, Chat, VS Code, and future SaaS
/// all render the same `EngineeringInsight` object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineeringInsight {
    /// Long-form answer for the intelligence query
    pub answer: String,
    /// Human-readable summary of the insight
    pub summary: String,
    /// Confidence score with explanations
    pub confidence: ConfidenceResult,
    /// Flattened evidence items for UI consumption
    pub evidence: Vec<EvidenceItem>,
    /// Actionable recommendations
    pub recommendations: Vec<String>,
    /// Warnings about missing data or risks
    pub warnings: Vec<String>,
    /// How this insight was produced
    pub mode: InferenceMode,
    /// Execution metadata
    pub metadata: InsightMetadata,
}
