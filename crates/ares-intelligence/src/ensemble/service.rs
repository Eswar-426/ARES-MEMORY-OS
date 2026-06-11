use super::aggregator::ResponseAggregator;
use super::validator::ResponseValidator;

pub struct EnsembleService {
    aggregator: ResponseAggregator,
    validator: ResponseValidator,
}

impl Default for EnsembleService {
    fn default() -> Self {
        Self::new(ResponseAggregator, ResponseValidator)
    }
}

impl EnsembleService {
    #[allow(dead_code)]
    pub fn new(aggregator: ResponseAggregator, validator: ResponseValidator) -> Self {
        Self {
            aggregator,
            validator,
        }
    }

    #[allow(dead_code)]
    pub fn resolve_conflict(&self, responses: &[String]) -> anyhow::Result<(String, f64)> {
        let score = self.validator.calculate_consensus_score(responses);
        let aggregated = self.aggregator.aggregate(responses)?;
        Ok((aggregated, score))
    }
}
