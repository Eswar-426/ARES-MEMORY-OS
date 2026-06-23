# ARES CLI Usability Report

## Overview
The CLI (`ares-cli`) provides the core entrypoint for users running ARES locally.

## Command Evaluation
### Discoverability
- **Good**: Uses `clap` for clear subcommands (`memory`, `governance`, `simulate`, `ingest`).

### Help Text
- **Needs Improvement**: The help text is extremely brief (e.g., "Repository Ingestion" for `ingest`). There are no usage examples, and it lacks clear onboarding instructions for new users.

### Error Messages
- **Critical Failure**: The CLI relies heavily on `.unwrap()` and `.expect()`. When a failure occurs (e.g., missing database, missing file, permission error), it crashes with a raw Rust stack trace instead of a human-readable error message.

### Onboarding Experience
- **Poor**: There is no initialization command or environment validation.

## Missing Commands
The following critical commands are missing:
- `ares doctor`: Required to diagnose the local environment, database status, and MCP connectivity.
- `ares status`: Required to see the current ingestion state, graph size, and project metrics.

## Conclusion
The CLI is functional but highly fragile. The usability is hampered by poor error handling and a lack of diagnostic tools.
