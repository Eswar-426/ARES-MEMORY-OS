# Requirement: Repository Scanner (Ingestion)
## ID: REQ-INGEST-001

The ingestion pipeline must begin with a file tree scan that discovers all source files, documentation, and configuration for processing.

The scanner must:
- Use walkdir for recursive traversal.
- Apply exclusion rules from .gitignore and built-in patterns.
- Return a Vec<PathBuf> of discovered files.
- Complete scanning within 1 second for repositories under 10k files.

Parent: REQ-MEMORY-004.

This requirement is implemented in crates/ares-ingestion/src/scanner.rs.
