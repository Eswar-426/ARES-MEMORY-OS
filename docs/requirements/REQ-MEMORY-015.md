# Requirement: Ownership Tracking
## ID: REQ-MEMORY-015

The system must extract and represent code ownership as first-class entities in the knowledge graph.

Ownership tracking must:
- Parse CODEOWNERS files at repository root and .github/CODEOWNERS.
- Parse ownership.md files.
- Create Owner nodes for each distinct owner identity.
- Create OwnedBy edges from CodeArtifact to Owner.
- Support wildcard pattern matching for default ownership.
- Support path-specific pattern matching for granular ownership.

This requirement is implemented in crates/ares-ingestion/src/extractors/ownership.rs and crates/ares-ingestion/src/graph.rs.
