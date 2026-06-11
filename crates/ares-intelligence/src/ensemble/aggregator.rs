use std::collections::HashMap;

pub struct ResponseAggregator;

impl Default for ResponseAggregator {
    fn default() -> Self {
        Self::new()
    }
}

impl ResponseAggregator {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }

    #[allow(dead_code)]
    pub fn aggregate(&self, responses: &[String]) -> anyhow::Result<String> {
        if responses.is_empty() {
            anyhow::bail!("No responses to aggregate");
        }

        // Basic voting: find the most frequent identical response (for strict consensus)
        let mut counts = HashMap::new();
        for res in responses {
            *counts.entry(res.clone()).or_insert(0) += 1;
        }

        let mut max_count = 0;
        let mut best_response = String::new();

        for (res, count) in counts {
            if count > max_count {
                max_count = count;
                best_response = res;
            }
        }

        Ok(best_response)
    }
}
