# Requirement: Streaming Ingestion
## ID: REQ-MEMORY-004

The system must ingest repository contents into the knowledge graph using a streaming pipeline architecture with bounded memory usage.

The streaming ingestion must:
- Scan the repository file tree.
- Extract intelligence from each file via language-specific extractors.
- Emit graph events (Node, Edge) through a streaming sink.
- Batch events into SQLite transactions (batch size: 2500).
- Maintain peak RSS below 100 MB for repositories up to 100k files.

This requirement is implemented in crates/ares-ingestion/src/graph.rs and crates/ares-cli/src/commands/ingest.rs.
