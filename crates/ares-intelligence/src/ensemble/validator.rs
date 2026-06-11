pub struct ResponseValidator;

impl Default for ResponseValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl ResponseValidator {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }

    #[allow(dead_code)]
    pub fn calculate_consensus_score(&self, responses: &[String]) -> f64 {
        if responses.is_empty() || responses.len() == 1 {
            return 1.0;
        }

        let first = &responses[0];
        let mut matching = 1;

        for res in responses.iter().skip(1) {
            // Highly naive consensus check: exact match
            if res == first {
                matching += 1;
            }
        }

        matching as f64 / responses.len() as f64
    }
}
