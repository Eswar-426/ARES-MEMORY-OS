# ARES Memory OS

Deterministic repository intelligence for AI coding agents. Zero LLM required.

---

## What It Does

ARES parses your repository into a queryable knowledge graph — AST relationships, git history, contributor ownership, and architectural decisions across 11 programming languages. When an AI agent asks *"What breaks if I change this trait?"*, ARES traverses the actual dependency graph and returns the exact blast radius — not a guess.

![ARES Knowledge Graph Explorer](docs/screenshots/graph-explorer.png)

---

## Installation

### VS Code / Antigravity IDE Extension

1. Download the latest `.vsix` from [GitHub Releases](https://github.com/Eswar-426/ARES-MEMORY-OS/releases)
2. Open VS Code / Antigravity IDE → `Extensions` → `...` → `Install from VSIX`
3. Select `ares-memory-vscode-0.1.0.vsix`

No Rust toolchain required. The extension bundles native binaries for Windows, macOS, and Linux.

---

## Quick Start

1. Open a repository in VS Code or Antigravity IDE.
2. The extension automatically detects the workspace and begins background ingestion (non-blocking).
3. Open the Output channel (`View` -> `Output` -> `ARES`) to monitor ingestion progress.
4. Interact via the Command Palette (`Ctrl+Shift+P`), ARES Query Panel, or let your AI agent call MCP tools directly.

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

---

## Complete MCP Tool Suite (39 Tools Total)

### 1. Understanding Code (7 Tools)

![Why Exists & Impact Analysis](docs/screenshots/why-exists.png)

| Tool | Description |
|------|-------------|
| `ares_why_exists` | Architectural reason a file/component exists with commit history |
| `ares_impact` | Blast radius — what breaks if this file or function changes |
| `ares_drift` | Measures divergence between written decisions and current code |
| `ares_who_owns` | Contributor percentage breakdown from git blame history |
| `ares_timeline` | Chronological commit evolution timeline for a file |
| `ares_compare` | Coupling score and shared dependencies between two files |
| `ares_simulate` | Deterministic simulation of dependency changes or removals |

### 2. Codebase Health (8 Tools)
| Tool | Description |
|------|-------------|
| `ares_health_check` | Composite repository health score (0-100) and gap breakdown |
| `ares_dead_code` | Detects unreferenced nodes and orphaned functions |
| `ares_architecture` | Overview of file/function counts and highest-coupled modules |
| `ares_scorecard` | Governance compliance scorecard across requirements, decisions, and ownership coverage |
| `ares_coverage` | Verification of decision and requirement coverage across code |
| `ares_compliance` | Evaluates code against architectural policies |
| `ares_dashboard` | Summary status of health metrics and recent trends |
| `ares_gaps` | Identifies undocumented code, stale decisions, or unassigned ownership |

### 3. Knowledge Management (8 Tools)
| Tool | Description |
|------|-------------|
| `ares_decisions` | Queries Architectural Decision Records (ADRs) linked to code |
| `ares_requirements` | Lists requirements linked to implementing functions |
| `ares_traceability` | Traces requirements down to exact file/function nodes |
| `ares_search` | Search files, functions, traits, or classes in the graph |
| `ares_record_decision` | Agent Write API: Create an ADR node linked to target files |
| `ares_record_requirement` | Agent Write API: Link a project requirement to code |
| `ares_annotate` | Agent Write API: Attach key-value annotations to any graph node |
| `ares_correct` | Agent Write API: Append correction metadata to any graph node |

### 4. Graph Exploration (6 Tools)

![Graph Explorer Visualization](docs/screenshots/graph-explorer-2.png)

| Tool | Description |
|------|-------------|
| `ares_graph_root` | Primary graph entry point for visual exploration |
| `ares_graph_neighbors` | Fetches direct incoming and outgoing graph connections |
| `ares_graph_shortest_path` | Finds shortest dependency path between any two nodes |
| `ares_graph_search` | Direct search over graph node labels and properties |
| `ares_graph_metadata` | Summary metadata for node and edge distribution |
| `ares_graph_statistics` | Quantitative graph density and degree statistics |

### 5. Session & Workspace Continuity (10 Tools)

![Session Briefing Context](docs/screenshots/briefing.png)
| Tool | Description |
|------|-------------|
| `ares_briefing` | Instant context briefing for new agent sessions |
| `ares_generate_context_file` | Automatically generates `.ares/CLAUDE.md` context summary |
| `ares_chat` | Interactive repository conversation engine |
| `ares_end_session` | Flushes session state and memory to database for handoff |
| `ares_session_context` | Retrieves history from previous agent sessions |
| `ares_workspace_pin` | Pins critical files to active agent context |
| `ares_workspace_bookmark` | Bookmarks key nodes for quick retrieval |
| `ares_workspace_navigate` | Navigates workspace bookmark locations |
| `ares_workspace_record_navigation` | Logs navigation trails during agent exploration |
| `ares_workspace_list` | Lists all pinned and bookmarked workspace items |

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

See [BENCHMARKS.md](BENCHMARKS.md) for full details.

---

## License

Distributed under the [MIT License](LICENSE). Copyright (c) 2026 Eswar-426.
