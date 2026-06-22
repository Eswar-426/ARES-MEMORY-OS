# Requirement: Governance Scorecard
## ID: REQ-GOV-003

The governance engine must produce a scorecard that summarizes repository memory health across all governance dimensions.

The scorecard must include:
- Requirement coverage percentage.
- Decision coverage percentage.
- Test coverage percentage.
- Ownership coverage percentage.
- Evidence coverage percentage.
- Overall compliance status.

Parent: REQ-MEMORY-011.

This requirement is implemented in crates/ares-governance/src/lib.rs and crates/ares-mcp/src/main.rs (ares_scorecard tool).
