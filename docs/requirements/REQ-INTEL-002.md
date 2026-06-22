# Requirement: Impact Analysis Query
## ID: REQ-INTEL-002

The system must answer "What breaks if this entity changes?" by performing downstream dependency analysis in the knowledge graph.

The query must:
- Identify all entities that directly depend on the target entity.
- Traverse transitive dependencies to project full blast radius.
- Return the affected entities as a structured report.
- Complete within 200 ms for any entity.

Parent: REQ-MEMORY-013.

This requirement is implemented in crates/ares-memory-intelligence/src/facade.rs (impact method) and crates/ares-knowledge-graph/src/impact.rs.
