# Requirement: Repository Memory Core
## ID: REQ-MEMORY-001

The system must provide a persistent, queryable knowledge graph that captures repository structure, relationships, and metadata.

The knowledge graph must:
- Store entities (CodeArtifact, Requirement, Decision, Architecture, Evidence, Owner) as typed nodes.
- Store relationships (ImplementedBy, Drives, ValidatedBy, OwnedBy, Supports, Contains) as typed edges.
- Support ACID transactions with rollback safety.
- Enforce referential integrity via foreign key constraints.
- Support bounded memory usage (< 100 MB peak RSS) for repositories up to 100k files.

This requirement is implemented in crates/ares-knowledge-graph/src/store.rs and crates/ares-knowledge-graph/src/models.rs.
