# Requirement: PR Impact Check
## ID: REQ-GOV-002

The governance engine must evaluate pull request changes against the current knowledge graph baseline and report coverage impacts.

PR checks must:
- Compare current graph state against a baseline snapshot.
- Detect new gaps introduced by the change.
- Detect requirements that lose coverage.
- Output results in SARIF format for CI integration.

Parent: REQ-MEMORY-011.

This requirement is implemented in crates/ares-governance/src/lib.rs and crates/ares-cli/src/commands/governance.rs.
