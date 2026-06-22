# Requirement: Governance Engine
## ID: REQ-MEMORY-011

The system must enforce governance policies and compliance checks against the knowledge graph.

The governance engine must:
- Define and enforce policy rules (e.g., every requirement must have implementation).
- Evaluate compliance for individual entities.
- Generate governance scorecards per project.
- Detect policy exemptions.
- Support PR-level impact checks against a baseline graph state.

This requirement is implemented in crates/ares-governance/src/lib.rs and crates/ares-governance/src/compliance_engine.rs.
