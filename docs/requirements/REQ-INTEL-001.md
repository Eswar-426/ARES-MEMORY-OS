# Requirement: Why Exists Query
## ID: REQ-INTEL-001

The system must answer "Why does this entity exist?" by traversing the knowledge graph from any CodeArtifact back to its originating Requirements, Decisions, and Evidence.

The query must:
- Return all linked Requirements (via ImplementedBy reverse traversal).
- Return all linked Decisions (via Drives reverse traversal).
- Return all linked Evidence (via Supports traversal from Decisions).
- Complete within 200 ms for any entity.

Parent: REQ-MEMORY-013.

This requirement is implemented in crates/ares-memory-intelligence/src/facade.rs (why method).
