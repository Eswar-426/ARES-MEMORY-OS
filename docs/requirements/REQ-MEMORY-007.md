# Requirement: VS Code Extension
## ID: REQ-MEMORY-007

The system must provide a VS Code extension that surfaces repository memory intelligence directly in the developer's IDE.

The extension must:
- Connect to the ares-mcp server via stdio transport.
- Provide commands for: Why Exists, Who Owns, Impact Analysis, Evolution Timeline.
- Display memory context inline with code navigation.
- Support file-save-triggered incremental ingestion.

This requirement is implemented in apps/vscode-extension/src/extension.ts.
