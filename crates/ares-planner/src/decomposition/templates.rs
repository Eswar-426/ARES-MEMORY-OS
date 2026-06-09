// Placeholder for predefined goal decomposition templates
// E.g., "Build Web App" -> [Setup Repo, Write Frontend, Write Backend, Deploy]

pub struct DecompositionTemplate {
    pub name: String,
    pub subgoals: Vec<String>,
}

impl DecompositionTemplate {
    pub fn new(name: impl Into<String>, subgoals: Vec<String>) -> Self {
        Self {
            name: name.into(),
            subgoals,
        }
    }
}
