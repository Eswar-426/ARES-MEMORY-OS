# Requirement: Markdown Intelligence Extraction
## ID: REQ-INGEST-003

The ingestion pipeline must extract intelligence from markdown documentation files.

Markdown intelligence must:
- Classify documents as Requirement, Decision, Architecture, Evidence, or Owner based on path and content patterns.
- Extract cross-references (REQ-xxx, ADR-xxx tokens) for automatic edge generation.
- Detect code path mentions to generate ImplementedBy and Drives edges.
- Generate ResultsIn edges (Requirement → Decision) when ADRs reference requirement IDs.
- Generate Supports edges (Evidence → Decision) when evidence documents reference ADR IDs.

Parent: REQ-MEMORY-004.

This requirement is implemented in crates/ares-ingestion/src/extractors/markdown.rs.
