# Architecture: Knowledge Graph Storage Layer
## ID: ARCH-001

The knowledge graph storage layer provides the persistent foundation for ARES.

Key responsibilities:
- Manage the SQLite database connection and pool.
- Enforce schema migrations and constraints.
- Expose CRUD operations for GraphEvent objects (Node/Edge).
- Buffer incoming streams and commit transactions.

This architecture satisfies ADR-002 and REQ-MEMORY-001.
This architecture governs crates/ares-knowledge-graph/ and crates/ares-store/.
