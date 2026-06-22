# Requirement: Repository Scanner
## ID: REQ-MEMORY-014

The system must scan repository file trees and produce a filtered list of files for ingestion.

The scanner must:
- Recursively walk the repository directory tree.
- Exclude build artifacts (target/, dist/, build/, node_modules/, coverage/).
- Exclude hidden directories (.git/, .ares/).
- Support incremental scanning (only changed files since last ingest).
- Produce canonical file paths normalized with forward slashes.

This requirement is implemented in crates/ares-scanner/src/lib.rs and crates/ares-ingestion/src/scanner.rs.
