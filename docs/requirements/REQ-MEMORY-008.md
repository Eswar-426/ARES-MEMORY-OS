# Requirement: CLI Interface
## ID: REQ-MEMORY-008

The system must provide a command-line interface for manual operators and CI/CD pipelines.

The CLI must support:
- `ares ingest .` — Full and incremental repository ingestion.
- `ares doctor` — System health check.
- `ares memory validate` — Memory certification validation.
- `ares memory export` — Graph snapshot export.
- `ares governance pr-check` — Pull request impact evaluation.
- `ares simulate` — Mutation analysis simulation.

This requirement is implemented in crates/ares-cli/src/main.rs and crates/ares-cli/src/commands/.

Supersedes: REQ-001.
