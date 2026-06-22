# Requirement: Streaming Persistence
## ID: REQ-INGEST-005

The ingestion pipeline must persist graph events to SQLite via batched streaming transactions.

Streaming persistence must:
- Accept GraphEvent items via a FnMut sink closure.
- Buffer events in batches of 2500.
- Commit each batch as a single SQLite transaction.
- Use `prepare_cached` for statement reuse across batches.
- Roll back failed batches without corrupting previous committed data.
- Never hold the full graph in memory simultaneously.

Parent: REQ-MEMORY-004.

This requirement is implemented in crates/ares-knowledge-graph/src/store.rs (upsert_batch) and crates/ares-cli/src/commands/ingest.rs.
