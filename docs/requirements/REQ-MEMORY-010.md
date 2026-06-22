# Requirement: Decision Intelligence
## ID: REQ-MEMORY-010

The system must extract, store, and reason about architectural decisions from repository documentation.

Decision intelligence must:
- Parse ADR documents from docs/decisions/ and recognized patterns (ADR-xxx, # Decision:).
- Create Decision nodes in the knowledge graph.
- Link Decisions to Requirements via ResultsIn edges.
- Link Decisions to Code via Drives edges.
- Link Decisions to Evidence via Supports edges.
- Support querying which decisions affect a given code artifact.

This requirement is implemented in crates/ares-decision-intelligence/src/lib.rs and crates/ares-ingestion/src/extractors/markdown.rs.
