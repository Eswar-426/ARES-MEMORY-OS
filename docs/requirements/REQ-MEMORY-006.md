# Requirement: MCP Server
## ID: REQ-MEMORY-006

The system must expose repository memory intelligence through a Model Context Protocol (MCP) server for IDE integration.

The MCP server must:
- Run as a stdio-based JSON-RPC server (ares-mcp).
- Expose tools: ares_why_exists, ares_who_owns, ares_impact, ares_evolution, ares_coverage, ares_compliance, ares_scorecard, ares_gaps, ares_simulate.
- Expose resources: memory://certification, memory://context/{id}, memory://summary/{id}.
- Achieve P95 latency < 200 ms for all tool invocations.
- Achieve 100% success rate across all registered repositories.

This requirement is implemented in crates/ares-mcp/src/main.rs.
