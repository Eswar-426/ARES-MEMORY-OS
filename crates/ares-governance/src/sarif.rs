use crate::models::{ComplianceResult, ViolationSeverity};
use serde_json::json;

pub struct GovernanceSarifExporter;

impl GovernanceSarifExporter {
    pub fn export_results(results: &[ComplianceResult]) -> serde_json::Value {
        let mut results_array = Vec::new();

        for res in results {
            for violation in &res.violations {
                let level = match violation.severity {
                    ViolationSeverity::Critical | ViolationSeverity::Error => "error",
                    ViolationSeverity::Warning => "warning",
                    ViolationSeverity::Info => "note",
                };

                let rule_id = violation.policy_name.clone();

                results_array.push(json!({
                    "ruleId": rule_id,
                    "level": level,
                    "message": {
                        "text": violation.reason
                    },
                    "locations": [{
                        "physicalLocation": {
                            "artifactLocation": {
                                "uri": format!("ares://node/{}", violation.node_id)
                            }
                        }
                    }]
                }));
            }
        }

        json!({
            "version": "2.1.0",
            "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
            "runs": [{
                "tool": {
                    "driver": {
                        "name": "ARES Governance Engine",
                        "informationUri": "https://ares.memoryos.org",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                },
                "results": results_array
            }]
        })
    }
}
