use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapRule {
    pub rule_id: String,
    pub target_type: String,      // "Capability", "Decision", etc.
    pub trigger_pattern: String,  // e.g., "tokio + axum"
    pub inferred_payload: String, // e.g., "Async Web Stack"
    pub confidence_score: f64,
}

pub trait RuleProvider: Send + Sync {
    fn load_rules(&self) -> Vec<BootstrapRule>;
}

pub struct BuiltInRules;

impl Default for BuiltInRules {
    fn default() -> Self {
        Self::new()
    }
}

impl BuiltInRules {
    pub fn new() -> Self {
        Self
    }
}

impl RuleProvider for BuiltInRules {
    fn load_rules(&self) -> Vec<BootstrapRule> {
        vec![
            BootstrapRule {
                rule_id: "builtin_async_stack".to_string(),
                target_type: "Decision".to_string(),
                trigger_pattern: "tokio|axum".to_string(),
                inferred_payload: "Adopt Async Rust Stack".to_string(),
                confidence_score: 0.9,
            },
            BootstrapRule {
                rule_id: "builtin_auth_capability".to_string(),
                target_type: "Capability".to_string(),
                trigger_pattern: "auth|jwt|oauth".to_string(),
                inferred_payload: "Authentication & Authorization".to_string(),
                confidence_score: 0.85,
            },
        ]
    }
}

pub struct YamlRules {
    pub file_path: String,
}

impl YamlRules {
    pub fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
        }
    }
}

impl RuleProvider for YamlRules {
    fn load_rules(&self) -> Vec<BootstrapRule> {
        if let Ok(content) = std::fs::read_to_string(&self.file_path) {
            #[derive(Deserialize)]
            struct YamlDoc {
                rules: Vec<BootstrapRule>,
            }
            if let Ok(doc) = serde_yaml::from_str::<YamlDoc>(&content) {
                return doc.rules;
            }
        }
        vec![]
    }
}
