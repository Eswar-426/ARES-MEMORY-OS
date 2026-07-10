# ARES MemoryOS for VS Code

![ARES Quality Report](https://img.shields.io/badge/ARES_Evaluation-96.4%25-brightgreen)
![Deterministic](https://img.shields.io/badge/Architecture-Deterministic-blue)
![Offline](https://img.shields.io/badge/Offline-First-brightgreen)
![7 Languages](https://img.shields.io/badge/Languages-7-blue)

Offline-first, deterministic repository intelligence for AI coding agents. No API keys required.

## What It Does

ARES parses your repository into a queryable knowledge graph — AST relationships, git history, ownership, and architectural decisions. When an AI agent asks *"What breaks if I change this trait?"*, ARES traverses the actual dependency graph and returns the exact blast radius — not a guess.

**This is fundamentally different from vector-search tools** (Mem0, Zep, Cognee). Those return semantically similar text. ARES returns graph-traversed facts.

## Installation

### From VS Code Marketplace (once published)
1. Open VS Code
2. Search "ARES MemoryOS"
3. Click Install

### From GitHub Releases (current method)
1. Go to [GitHub Releases](https://github.com/Eswar-426/ARES-MEMORY-OS/releases)
2. Download `ares-memory-vscode-0.1.0.vsix`
3. In VS Code: `Extensions` → `...` → `Install from VSIX`

The extension bundles native binaries for Windows, macOS (ARM + x64), and Linux. No Rust toolchain required.

## Quick Start

1. Open a repository in VS Code
2. Run **ARES: Ingest Repository** from the Command Palette (`Ctrl+Shift+P`)
3. Wait for ingestion to complete (1–15 minutes depending on repo size)
4. Open the **ARES Chat** webview from the sidebar
5. Ask architecture questions directly

## Features

### Intelligence Engines (graph traversal, zero LLM)
| Tool | What It Does | Example Output |
|------|-------------|----------------|
| `ares_why_exists` | Finds why a file exists | *"Imported from private SVN repository (created from r. 8825) by Adrian Holovaty"* |
| `ares_impact` | Blast radius — what breaks if this file changes | *"Risk: HIGH, Affected modules: 12"* |
| `ares_drift` | Has this file drifted from its documented architecture | *"MEDIUM (score: 45), No violations"* |
| `ares_traceability` | Trace a requirement to implementing functions | *"REQ-12 → DEC-JWT → func:validate_jwt"* |
| `ares_simulate` | "What if I remove this?" — deterministic simulation | *"Removing this entity impacts 0 files"* |

### Query Tools (direct graph reads)
| Tool | What It Does |
|------|-------------|
| `ares_who_owns` | Contributor percentages from git blame data |
| `ares_search` | Exact-match search on file/function/class names |
| `ares_timeline` | Chronological commit history for a file |
| `ares_compare` | Coupling score and shared dependencies between two files |
| `ares_architecture` | Repository overview: file/function counts, top coupled files |
| `ares_decisions` | Architectural Decision Records linked to files |
| `ares_requirements` | Requirements linked to implementing files |
| `ares_health_check` | Gap detection + health score (0–100) |

### Write Tools (agent memory persistence)
| Tool | What It Does |
|------|-------------|
| `ares_record_decision` | Create an architectural decision node linked to files |
| `ares_record_requirement` | Link a requirement to implementing files |
| `ares_annotate` | Add a key-value annotation to any node |
| `ares_correct` | Append a correction record to any node |

### Session Tools (agent continuity)
| Tool | What It Does |
|------|-------------|
| `ares_session_context` | Retrieve last 3 agent sessions for context injection |
| `ares_end_session` | Flush current session data to DB for next session |

### CLI Commands
| Command | What It Does |
|---------|--------------|
| `ares ingest .` | Full scan: AST, file inventory, git history, blame |
| `ares doctor` | Database integrity check |
| `ares overview` | Generate `.ares/system_overview.md` for agent context |
| `ares health` | Print health check report to terminal |
| `ares compact` | Run VACUUM + ANALYZE to reduce DB size |

## Agent Integration

ARES exposes all tools via MCP. Any agent that supports MCP can connect:

```json
{
  "mcpServers": {
    "ares": {
      "command": "path/to/ares-mcp",
      "cwd": "/path/to/your/repo",
      "args": []
    }
  }
}
```

No `project_id` configuration needed. The server resolves it from the workspace CWD automatically.

### Claude Code Example

```json
{"command": "ares_impact", "arguments": {"file_path": "django/http/request.py"}}
```

### Cursor / Cline Example

```json
{"command": "ares_health_check", "arguments": {}}
```

## Tested Results

Tested on django-full2 (7,088 files, 500 commits):

| Metric | Result |
|-------|--------|
| Files parsed | 7,088 |
| Nodes in graph | 61,901 |
| Edges in graph | 112,944 |
| Why Exists accuracy | EXCELLENT — cites actual commit messages and SVN history |
| Impact radius | WORKING — correctly returns 12 dependents for core files |
| Health check | WORKING — returns gap counts and accurate ~9 score for django |
| Query performance | < 200ms for most queries |
| DB size (100K LOC) | 59.3 MB (tokio) |

## Supported Languages

Rust, TypeScript, Python, Go, JavaScript, Java, C#, C/C++, Ruby

## Architecture Overview

```
ares-mcp (MCP Server, thin transport)
  └── ares-repository-intelligence (deterministic pipeline)
        ├── EvidenceService (ONLY component that touches DB)
        ├── InferenceRegistry (maps QueryType → Generator)
        └── Generators (WhyExists, Impact, Drift, Traceability)
```

No LLM in the query path. AI is only used for the optional `ares_record_decision` workflow.

## Screenshots

<!-- TODO: Add screenshots here after capturing from VS Code -->

## License

MIT
