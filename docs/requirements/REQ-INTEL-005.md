# Requirement: Self-Hosting Validation
## ID: REQ-INTEL-005

ARES must be capable of validating its own repository memory quality by querying its own knowledge graph.

Self-hosting validation must:
- Measure requirement-to-code coverage for the ARES repository.
- Measure requirement-to-test coverage for the ARES repository.
- Measure decision-to-code linkage for the ARES repository.
- Measure decision-to-evidence linkage for the ARES repository.
- Report files without rationale, ownership, and tests.
- Generate reports/validation/self_hosting_readiness.md with real metrics.

Parent: REQ-MEMORY-009.

This requirement is implemented in scripts/self_hosting.py.
