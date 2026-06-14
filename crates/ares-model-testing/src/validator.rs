use crate::scenarios::Scenario;

pub struct ContinuityValidator;

impl ContinuityValidator {
    /// Evaluates the model's response against the expected keywords.
    /// Returns a score between 0.0 and 100.0
    pub fn evaluate(response: &str, scenario: &Scenario) -> f64 {
        let text = response.to_lowercase();
        let mut matches = 0;

        for keyword in &scenario.expected_keywords {
            if text.contains(&keyword.to_lowercase()) {
                matches += 1;
            }
        }

        if scenario.expected_keywords.is_empty() {
            return 100.0;
        }

        (matches as f64 / scenario.expected_keywords.len() as f64) * 100.0
    }
}
