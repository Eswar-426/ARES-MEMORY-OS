use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardrailPolicy {
    pub allow_destructive_actions: bool,
    pub max_cost: Option<f64>,
    pub required_approvals: usize,
    pub banned_commands: Vec<String>,
}

impl Default for GuardrailPolicy {
    fn default() -> Self {
        Self {
            allow_destructive_actions: false,
            max_cost: Some(10.0),  // $10 budget limit
            required_approvals: 1, // Require manual approval for any critical plan
            banned_commands: vec!["rm -rf /".into(), "mkfs".into(), "sudo".into()],
        }
    }
}
