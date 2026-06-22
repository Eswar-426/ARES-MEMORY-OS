# Requirement: Context Assembly
## ID: REQ-MEMORY-013

The system must assemble rich contextual information for any entity in the knowledge graph.

Context assembly must:
- Aggregate related requirements, decisions, evidence, and ownership for any given entity.
- Support "Why Exists" queries that explain the rationale chain behind a code artifact.
- Support "Who Owns" queries that identify responsible parties.
- Support "Impact" queries that project downstream blast radius of changes.
- Produce token-efficient summaries suitable for LLM consumption.

This requirement is implemented in crates/ares-context-generator/src/lib.rs and crates/ares-memory-intelligence/src/facade.rs.
