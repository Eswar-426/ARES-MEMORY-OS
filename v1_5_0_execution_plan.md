# ARES v1.5.0 Execution Plan

## Objective
Execute the minimum set of changes required to make ARES testable by users and agents in real repositories without expanding scope. No new features will be built.

## 1. Must Fix Before Testing
These items will outright break the developer experience or agent integrations if left unresolved.

- **Remove `unwrap()` and `expect()` from CLI and Ingestion Engine**: Refactor all `.unwrap()` calls to bubble up `AresError` and print formatted, human-readable error messages to stderr.
- **Fix MCP Serialization Masking**: Refactor `ares-mcp` to return proper JSON-RPC error codes rather than returning string literals like `"Failed to serialize"` or empty structures when serialization fails.
- **Implement Extension Ingestion Command**: Add an `ares.ingest` command to the VS Code extension that executes the CLI ingestion process and streams output.
- **Wire Extension to MCP**: Integrate an MCP client into the VS Code extension so it can communicate with `ares-mcp` to support "Ask why" and "Ask impact" queries.

## 2. Should Fix Before Testing
These items significantly degrade usability but have workarounds.

- **Add `ares doctor` CLI command**: Implement a diagnostic command that checks for the `.ares` directory, validates the knowledge graph schema, and checks if the MCP daemon can be started.
- **Improve CLI Help Text**: Expand `clap` documentation in `ares-cli` with concrete examples (e.g., `ares ingest .`, `ares memory validate --ci`).

## 3. Nice to Have
Enhancements that would polish the experience but are not strictly blocking.

- **Add `ares status` CLI command**: To quickly view ingestion stats (nodes/edges) without re-running ingest.
- **VS Code Extension Hover Provider**: Implement a simple hover provider that queries the MCP server for "Why it exists" when hovering over a symbol, rather than requiring a manual command execution.

## 4. Safe to Defer Until After Validation
Items that fall outside the MVP workflow or represent significant scope expansion.

- **Full VS Code Extension Tree View**: Building a full repository graph tree view in the sidebar.
- **Advanced Code Actions / Diagnostics**: Inline diagnostics for governance compliance.
- **Dynamic Port Resolution for MCP**: The extension can continue using a hardcoded or configured port/stdio for initial testing, deferring robust auto-discovery.
