# Requirement: Incremental Capture
## ID: REQ-INGEST-004

The system must support incremental ingestion triggered by file changes, without requiring a full repository re-scan.

Incremental capture must:
- Accept a list of changed file paths.
- Filter the file scan to only process changed files.
- Upsert modified nodes and edges without duplicating existing data.
- Support file-watcher integration for continuous capture.

Parent: REQ-MEMORY-004.

This requirement is implemented in crates/ares-ingestion/src/graph.rs (set_incremental_files) and crates/ares-cli/src/commands/ingest.rs.
