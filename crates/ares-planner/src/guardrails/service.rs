use crate::guardrails::policy::GuardrailPolicy;
use crate::guardrails::validator::GuardrailValidator;
use crate::models::candidate::PlanCandidate;
use ares_core::AresError;

#[derive(Default)]
pub struct GuardrailService {
    policy: GuardrailPolicy,
}

impl GuardrailService {
    pub fn new(policy: GuardrailPolicy) -> Self {
        Self { policy }
    }

    pub fn enforce_policy(&self, candidate: &PlanCandidate) -> Result<(), AresError> {
        GuardrailValidator::validate_candidate(&self.policy, candidate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ares_core::id::PlanId;

    #[test]
    fn test_guardrail_service_enforcement() {
        let mut policy = GuardrailPolicy::default();
        policy.banned_commands.push("sudo".to_string());

        let service = GuardrailService::new(policy);

        let bad_candidate =
            PlanCandidate::new_test(PlanId::new(), r#"{"command": "sudo rm -rf"}"#.to_string());
        let good_candidate =
            PlanCandidate::new_test(PlanId::new(), r#"{"command": "echo hello"}"#.to_string());

        assert!(service.enforce_policy(&bad_candidate).is_err());
        assert!(service.enforce_policy(&good_candidate).is_ok());
    }
}
