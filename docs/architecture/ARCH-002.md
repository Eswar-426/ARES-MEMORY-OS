# Architecture: Streaming Ingestion Pipeline
## ID: ARCH-002

The streaming ingestion pipeline defines how ARES converts raw repository files into a structured knowledge graph without unbounded memory growth.

Key responsibilities:
- Orchestrate the repository file scan (via ares-scanner).
- Stream files through language-specific and markdown intelligence extractors.
- Produce an iterator of `GraphEvent` records.
- Pass events to the knowledge graph store for batched transaction writes.

This architecture satisfies ADR-003, ADR-010, and REQ-MEMORY-004.
This architecture governs crates/ares-ingestion/.
