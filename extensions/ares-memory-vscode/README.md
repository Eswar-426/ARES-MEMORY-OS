# ARES Memory OS for VS Code

Deterministic repository intelligence for AI coding agents. Zero LLM required.

---

## What It Does

ARES parses your repository into a queryable knowledge graph — AST relationships, git history, contributor ownership, and architectural decisions across 11 programming languages. When an AI agent asks *"What breaks if I change this trait?"*, ARES traverses the actual dependency graph and returns the exact blast radius — not a guess.

---

## Installation

### From VS Code Marketplace (once published)
1. Open VS Code or Antigravity IDE
2. Search "ARES Memory OS"
3. Click Install

### From GitHub Releases (current method)
1. Go to [GitHub Releases](https://github.com/Eswar-426/ARES-MEMORY-OS/releases)
2. Download `ares-memory-vscode-0.1.0.vsix`
3. In VS Code: `Extensions` → `...` → `Install from VSIX`

The extension bundles native binaries for Windows, macOS, and Linux. No Rust toolchain required.

---

## Quick Start

1. Open a repository in VS Code or Antigravity IDE.
2. The extension automatically activates and begins background ingestion (non-blocking).
3. Open the Output channel (`View` → `Output` → `ARES`) to monitor ingestion progress.
4. Interact via the Command Palette (`Ctrl+Shift+P`), ARES Query Panel, or let your AI agent call MCP tools directly.

---

## Activation

The extension activates on any workspace open (`activationEvents: ["*"]`). If `ares.autoIngestOnOpen` is enabled (default), ingestion starts automatically in the background without blocking the editor.

**Note:** There is no status bar indicator in v0.1.0. Monitor ingestion via the Output channel.

---

## Commands (24 Registered)

### Repository Operations
| Command | Description |
|---------|-------------|
| `ARES: Ingest Repository` | Full scan: AST, file inventory, git history, blame |
| `ARES: Rebuild Database` | Delete and rebuild the knowledge graph from scratch |
| `ARES: Doctor` | Run database integrity checks |
| `ARES: Compact Database` | Run VACUUM + ANALYZE to reduce DB size |

### Intelligence Queries
| Command | Description |
|---------|-------------|
| `ARES: Why Exists` | Why does this file exist? Creation context and evolution |
| `ARES: Impact Analysis` | What breaks if this file changes? Blast radius |
| `ARES: Drift Analysis` | Has this file drifted from its documented purpose? |
| `ARES: Who Owns` | Contributor percentages from git blame |
| `ARES: Traceability Analysis` | Trace requirements to implementing functions |
| `ARES: Coverage Analysis` | Decision and requirement coverage metrics |
| `ARES: Simulate Change` | "What if I remove this?" simulation |
| `ARES: Architecture Map` | Repository overview and coupling analysis |

### Health & Governance
| Command | Description |
|---------|-------------|
| `ARES: Health Check` | Composite health score (0-100) and gap breakdown |
| `ARES: Scorecard` | Governance compliance across requirements, decisions, ownership |
| `ARES: Gaps` | Identify undocumented code, stale decisions, unassigned ownership |
| `ARES: Dashboard` | Summary status of health metrics and trends |
| `ARES: Run Diagnostics` | Extension diagnostics and connection status |

### Visualization
| Command | Description |
|---------|-------------|
| `ARES: Open Graph Explorer` | Interactive graph visualization |
| `ARES: Graph Explorer` | Alternative graph view |
| `ARES: Query Panel` | Open the ARES query panel |
| `ARES: Open Chat` | Interactive repository conversation |

### Agent Write Operations
| Command | Description |
|---------|-------------|
| `ARES: Record Decision` | Create an architectural decision linked to files |

### CLI Outputs
| Command | Description |
|---------|-------------|
| `ARES: Overview` | Generate `.ares/system_overview.md` for agent context |
| `ARES: Health Report` | Print health check to terminal |

### Context Menu (Right-Click)
- **Why Exists** — on files
- **Impact Analysis** — on files
- **Open Graph** — on files and folders
- **Who Owns** — on files and folders
- **Drift Analysis** — on files and folders
- **Simulate Change** — on files and folders

---

## Settings

| Setting | Default | Description |
|---------|---------|-------------|
| `ares.autoIngestOnOpen` | `true` | Automatically ingest when opening a repository |
| `ares.generateContextFile` | `true` | Generate `.ares/CLAUDE.md` after ingestion |
| `ares.telemetry.enabled` | `false` | Disable all telemetry (no data leaves your machine) |
| `ares.mcpPath` | `""` | Custom path to ares-mcp binary (auto-detected if empty) |
| `ares.cliPath` | `""` | Custom path to ares CLI binary (auto-detected if empty) |

---

## Supported Languages (11 Total)

ARES includes native Tree-sitter AST extractors for:
- **Rust** (`.rs`)
- **Python** (`.py`, `.pyw`)
- **TypeScript** (`.ts`, `.tsx`)
- **JavaScript** (`.js`, `.jsx`)
- **Go** (`.go`)
- **Java** (`.java`)
- **C / C++** (`.c`, `.cpp`, `.h`, `.hpp`)
- **C#** (`.cs`)
- **PHP** (`.php`)
- **Ruby** (`.rb`)
- **Kotlin** (`.kt`, `.kts`)

Files in unsupported languages are still indexed for git history and file-level relationships.

---

## Complete MCP Tool Suite (39 Tools)

### Understanding Code (7)
`ares_why_exists` · `ares_impact` · `ares_drift` · `ares_who_owns` · `ares_timeline` · `ares_compare` · `ares_simulate`

### Codebase Health (8)
`ares_health_check` · `ares_dead_code` · `ares_architecture` · `ares_scorecard` · `ares_coverage` · `ares_compliance` · `ares_dashboard` · `ares_gaps`

### Knowledge Management (8)
`ares_decisions` · `ares_requirements` · `ares_traceability` · `ares_search` · `ares_record_decision` · `ares_record_requirement` · `ares_annotate` · `ares_correct`

### Graph Exploration (6)
`ares_graph_root` · `ares_graph_neighbors` · `ares_graph_shortest_path` · `ares_graph_search` · `ares_graph_metadata` · `ares_graph_statistics`

### Session & Workspace (10)
`ares_briefing` · `ares_generate_context_file` · `ares_chat` · `ares_end_session` · `ares_session_context` · `ares_workspace_pin` · `ares_workspace_bookmark` · `ares_workspace_navigate` · `ares_workspace_record_navigation` · `ares_workspace_list`

---

## Agent Integration

ARES exposes all tools via MCP. Any MCP-compatible agent can connect:

```json
{
  "mcpServers": {
    "ares": {
      "command": "path/to/ares-mcp",
      "args": ["--workspace", "/path/to/your/repo"]
    }
  }
}
```

The extension automatically configures MCP for Claude Code, Cursor, Windsurf, Codex, and Claude Desktop when activated.

---

## Measured Benchmarks

| Repository | Files | Functions | Nodes | Edges | DB Size | Query p95 |
|------------|-------|-----------|-------|-------|---------|-----------|
| tokio | 848 | 1,932 | 24,433 | 65,490 | 53.5 MB | < 15ms |
| django | 14,291 | 46,863 | 119,221 | 174,968 | 209.6 MB | < 25ms |
| react | 7,006 | 1,169 | 42,286 | 71,372 | 82.4 MB | < 20ms |
| go | 6,120 | 41,500 | 172,235 | 210,592 | 184.0 MB | < 22ms |
| vscode | 12,400 | 85,200 | 453,576 | 605,409 | 556.3 MB | < 35ms |

See [BENCHMARKS.md](https://github.com/Eswar-426/ARES-MEMORY-OS/blob/main/BENCHMARKS.md) for full details.

---

## License

MIT — [Eswar-426](https://github.com/Eswar-426)
