# Requirement: Policy Enforcement
## ID: REQ-GOV-001

The governance engine must define and enforce repository-level policies that ensure memory quality standards.

Policies include:
- Every Requirement must have at least one ImplementedBy edge.
- Every Decision must have at least one Drives edge.
- Every CodeArtifact must have an OwnedBy edge.
- Critical subsystems must have ValidatedBy edges.

Parent: REQ-MEMORY-011.

This requirement is implemented in crates/ares-governance/src/compliance_engine.rs.
