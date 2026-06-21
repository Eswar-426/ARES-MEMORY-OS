# ARES Extension Gap Analysis

## Current Implementation State
Based on the audit of `extensions/ares-memory-vscode`, the current extension is a very early MVP.

### What Already Exists
- **Commands**:
  - `ares.saveProjectMemory`
  - `ares.restoreProjectMemory`
  - `ares.generateSnapshot`
  - `ares.searchMemory`
  - `ares.projectStatus`
  - `ares.copyContextToClipboard`
- **Sidebar & Panels**:
  - `aresMemorySidebar` webview.
- **Missing Elements**:
  - NO tree views.
  - NO diagnostics.
  - NO code actions.
  - NO hover providers.

## Gap Analysis for Minimum Workflow

### Open repository
- **Status**: Supported (natively by VS Code).

### Run ingest
- **Status**: **Missing**.
- **Gap**: There is no command like `ares.ingest` exposed in the extension. The user must manually run `ares ingest` via terminal.

### Ask why code exists
- **Status**: **Missing**.
- **Gap**: The extension does not connect to the MCP server or provide an interactive chat/hover to query the `ares_why_exists` tool.

### Ask impact
- **Status**: **Missing**.
- **Gap**: No contextual menu or command to invoke `ares_impact` on a selected file or function.

### Ask coverage
- **Status**: **Missing**.
- **Gap**: No integration with the `ares_coverage` tool.

### Ask simulation
- **Status**: **Missing**.
- **Gap**: No UI or command to trigger the simulation engine (`ares_simulate`).

## Conclusion
The extension is currently a skeleton wrapper around a few local HTTP API calls to `localhost:3000`. It entirely lacks the code-level integrations (Hover, Diagnostics, Code Actions) and MCP bindings required for the minimum workflow.
