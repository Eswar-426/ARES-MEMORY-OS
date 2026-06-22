# Requirement: Dependency Extraction
## ID: REQ-INGEST-002

The ingestion pipeline must extract dependency relationships from source code files.

Dependency extraction must support:
- Rust: `use`, `extern crate`, Cargo.toml dependencies.
- TypeScript/JavaScript: `import`, `require`, package.json dependencies.
- Python: `import`, `from ... import`, requirements.txt/pyproject.toml.

Each extracted dependency must produce a DependsOn edge and a synthesized dependency node if none exists.

Parent: REQ-MEMORY-004.

This requirement is implemented in crates/ares-ingestion/src/extractors/rust.rs and crates/ares-ingestion/src/extractors/typescript.rs.
