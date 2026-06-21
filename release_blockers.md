# ARES V1.5.0 Release Blockers

## Critical Blockers
1. **CLI Panics on Failure**: `unwrap()` and `expect()` usage in `ares-cli` and `ares-ingestion` causes hard crashes instead of graceful exits (e.g., when a file cannot be written or `SystemTime` fails). This violates production reliability standards.
2. **Missing Extension Workflow**: The VS Code extension lacks the ability to run repository ingestion (`ares.ingest`) and cannot execute MCP tools (`ares_why_exists`, `ares_impact`). The minimum required developer workflow is broken.
3. **Masked MCP Serialization Errors**: `ares-mcp` endpoints return raw strings (`"Failed to serialize"`) or empty defaults (`unwrap_or_default()`) when JSON serialization fails, violating the MCP JSON-RPC protocol expectations and breaking clients.

## High Blockers
1. **Lack of Diagnostics**: There is no `ares doctor` command to help users diagnose missing dependencies, bad configurations, or daemon connectivity issues.
2. **Hardcoded Endpoints**: The VS Code extension has a hardcoded dependency on `http://localhost:3000/api/v1` instead of dynamically resolving the local daemon or MCP stdio transport.

## Medium Blockers
1. **Help Text Clarity**: CLI `--help` messages are too brief and lack examples for onboarding new users.

## Low Blockers
1. **Missing `ares status`**: While `ares status` is missing, users can manually check the `.ares` folder as a workaround.
