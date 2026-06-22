# Architecture: MCP Protocol Server
## ID: ARCH-003

The MCP architecture dictates how external IDEs interact with ARES intelligence over stdio.

Key responsibilities:
- Serve JSON-RPC payload framing over standard input/output streams.
- Define tools (ares_why_exists, ares_impact) and execute them against the knowledge graph.
- Expose resource schemas and context data.
- Integrate with incremental file watcher payloads for real-time memory syncing.

This architecture satisfies ADR-008 and REQ-MEMORY-006.
This architecture governs crates/ares-mcp/.
