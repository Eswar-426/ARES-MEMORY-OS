# ARES Error Handling Report

## Audit Scope
Audited `ares-mcp`, `ares-cli`, and `ares-ingestion`.

## Critical Issues: Panics and Stack Traces

### `crates/ares-cli`
Users will receive raw Rust stack traces in the following locations due to unhandled `.unwrap()` calls:
- `crates/ares-cli/src/commands/memory.rs`
  - Line 35: `std::fs::write(...).unwrap()`
  - Line 43: `serde_json::to_string_pretty(...).unwrap()`
  - Line 122: `serde_json::to_string_pretty(...).unwrap()`
- `crates/ares-cli/src/commands/governance.rs`
  - Line 99: `base_snapshot.unwrap()`

### `crates/ares-ingestion`
- `crates/ares-ingestion/src/graph.rs`
  - Line 29: `SystemTime::now().duration_since(UNIX_EPOCH).unwrap()`

### `crates/ares-mcp`
- `crates/ares-mcp/src/main.rs`
  - Line 47: `std::env::current_dir().expect("Cannot determine current directory")`

## Silent Failures & Masked Errors

### Invalid JSON Responses / Silent Defaults
In the MCP endpoints, serialization errors or missing data are often masked, returning invalid string responses or empty defaults that break client applications:
- `crates/ares-mcp/src/main.rs`
  - Line 99, 114, 196, 212: `unwrap_or_else(|_| "Failed to serialize".to_string())` - Returns a plain string instead of a valid JSON error payload.
  - Line 136, 154, 183, 240, 252, 264, 284: `unwrap_or_default()` - Returns empty strings or `{}` instead of reporting the failure to the MCP client.

## Conclusion
ARES currently relies on happy-path execution. Any deviation (missing files, bad permissions, serialization issues) results in hard panics or malformed JSON payloads. These must be replaced with `Result` bubbling and proper user-facing error formatting.
