# ARES MemoryOS

Deterministic repository intelligence for AI coding agents. Zero LLM required.

## What It Does

ARES parses your repository into a queryable knowledge graph — AST relationships, git history, ownership, and architectural decisions. When an AI agent asks *"What breaks if I change this trait?"*, ARES traverses the actual dependency graph and returns the exact blast radius — not a guess.

## Installation

**VS Code Marketplace** (once published):

1. Open VS Code
2. Search "ARES MemoryOS"
3. Click Install

**Manual install:**

1. Download the latest `.vsix` from [GitHub Releases](https://github.com/Eswar-426/ARES-MEMORY-OS/releases)
2. `Extensions` → `...` → `Install from VSIX`

No Rust toolchain required. The extension bundles native binaries for Windows, macOS (ARM + x64), and Linux.

## Quick Start

1. Open a repository in VS Code
2. Run **ARES: Ingest Repository** from the Command Palette (`Ctrl+Shift+P`)
3. Wait for ingestion to complete (1-15 minutes depending on repo size)
4. Ask questions via the ARES query panel or let your AI agent call MCP tools directly

## Example Queries (Tested on Django)

These are real outputs from the django-full2 repository (7,088 files, 500 commits):

**Why does this module exist?**
```
Introduced for: MERGED MAGIC-REMOVAL BRANCH TO TRUNK.
This change is highly backwards-incompatible and should only be applied after reviewing 
the migration guide.
```

**What breaks if I change this file?**
```
Risk: HIGH
Affected modules: 12
- django/http/response.py
- django/http/request.py
- django/core/handlers/wsgi.py
- django/middleware/common.py
- ...
```

**Who owns this file?**
```
django-bot: 66%
Tim Graham: 13%
Other: 21%
```

**How are these two files related?**
```
Coupling score: 42
Relationship: loosely coupled
Shared dependencies: io, urllib.parse, django.utils.http
```

## Available Tools

### Intelligence Engines (graph traversal, zero LLM)
| Tool | What It Does |
|------|-------------|
| `ares_why_exists` | Finds the architectural reason a file exists |
| `ares_impact` | Blast radius — what breaks if this file changes |
| `ares_drift` | Has this file drifted from its documented architecture |
| `ares_traceability` | Trace a requirement down to implementing functions |
| `ares_simulate` | "What if I remove this?" — deterministic impact simulation |

### Query Tools (direct graph reads)
| Tool | What It Does |
|------|-------------|
| `ares_who_owns` | Contributor percentages from git blame data |
| `ares_search` | Exact-match search on file/function/class names |
| `ares_timeline` | Chronological commit history for a file |
| `ares_compare` | Coupling score and shared dependencies between two files |
| `ares_architecture` | Repository overview: file/function counts, top coupled files |
| `ares_decisions` | Architectural Decision Records linked to files |
| `requirements` | Requirements linked to implementing files |
| `ares_health_check` | Gap detection + health score (0-100) |

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
| `ares ingest .` | Full repository scan: AST, file inventory, git history, blame |
| `ares doctor` | Database integrity check |
| `ares overview` | Generate `.ares/system_overview.md` for agent context |
| `ares compact` | Run VACUUM + ANALYZE to reduce DB size |
| `ares health` | Run health check and print report to terminal |

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

The agent receives `project_id` automatically from the CWD. No project path configuration needed.

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

## License

MIT
