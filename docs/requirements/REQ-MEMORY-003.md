# Requirement: Knowledge Gap Detection
## ID: REQ-MEMORY-003

The system must deterministically detect and report knowledge gaps in the repository memory graph.

Knowledge gaps include:
- Requirements without implementation (RequirementWithoutImplementation).
- Requirements without tests (RequirementWithoutTests).
- Decisions without implementation (DecisionWithoutImplementation).
- Decisions without evidence (DecisionWithoutEvidence).
- Code without rationale (CodeWithoutRationale).
- Code without tests (CodeWithoutTests).
- Code without ownership (CodeWithoutOwnership).

Gap detection must run during every full ingest and every incremental ingest.

This requirement is implemented in crates/ares-gap-engine/src/lib.rs and crates/ares-ingestion/src/gap_generator.rs.
