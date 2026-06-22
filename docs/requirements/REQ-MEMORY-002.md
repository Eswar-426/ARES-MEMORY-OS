# Requirement: Traceability Engine
## ID: REQ-MEMORY-002

The system must provide bidirectional traceability between requirements, decisions, code, tests, and evidence.

The traceability engine must:
- Link Requirements to Code via ImplementedBy edges.
- Link Decisions to Code via Drives edges.
- Link Code to Tests via ValidatedBy edges.
- Link Evidence to Decisions via Supports edges.
- Support querying the full trace chain: Requirement → Decision → Code → Test → Evidence.
- Compute traceability completeness as a percentage metric.

This requirement is implemented in crates/ares-traceability/src/lib.rs.
