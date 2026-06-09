use crate::guardrails::policy::GuardrailPolicy;
use crate::models::candidate::PlanCandidate;
use ares_core::AresError;

pub struct GuardrailValidator;

impl GuardrailValidator {
    pub fn validate_candidate(
        policy: &GuardrailPolicy,
        candidate: &PlanCandidate,
    ) -> Result<(), AresError> {
        // In the future: Parse candidate.dag_json and check for policy violations.
        // For example, looking for banned commands in bash task nodes.

        let dag_json = &candidate.dag_json;

        for banned in &policy.banned_commands {
            if dag_json.contains(banned) {
                return Err(AresError::validation(format!(
                    "Plan candidate violates safety guardrail: contains banned command '{}'",
                    banned
                )));
            }
        }

        Ok(())
    }

    pub fn validate_goal(&self, _goal: &crate::models::goal::Goal) -> Result<(), AresError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ares_core::id::PlanId;

    #[test]
    fn test_guardrail_validator_rejects_banned_commands() {
        let mut policy = GuardrailPolicy::default();
        policy.banned_commands.push("rm -rf /".to_string());

        let candidate = PlanCandidate::new_test(
            PlanId::new(),
            r#"{"nodes": [{"command": "rm -rf /"}]}"#.to_string(),
        );

        let result = GuardrailValidator::validate_candidate(&policy, &candidate);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("banned command 'rm -rf /'"));
    }

    #[test]
    fn test_guardrail_validator_allows_safe_commands() {
        let mut policy = GuardrailPolicy::default();
        policy.banned_commands.push("rm -rf /".to_string());

        let candidate = PlanCandidate::new_test(
            PlanId::new(),
            r#"{"nodes": [{"command": "ls -al"}]}"#.to_string(),
        );

        let result = GuardrailValidator::validate_candidate(&policy, &candidate);
        assert!(result.is_ok());
    }
}
