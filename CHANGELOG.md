# Changelog

All notable changes to ARES Memory OS will be documented in this file.

## [0.1.0] - 2026-07-24

### Added
- **Initial release of ARES Memory OS** — repository intelligence for AI agents
- **21 MCP tools** for deterministic, evidence-based code analysis
- **Why Exists**: creation commit, architectural role, evolution history
- **Impact Analysis**: blast radius, risk level, execution flows
- **Briefing**: instant project context for new agent sessions with session handoff
- **Health Check**: composite score with hotspots, gaps, score breakdown
- **Dead Code Detection**: file and function level with exclusion filters
- **Drift Detection**: decision-to-code divergence scoring
- **Decision Recording**: agent write API with provenance tracking
- **Hidden Coupling Detection**: co-change analysis in architecture
- **Decision Confidence Decay**: staleness scoring based on commit activity
- **Graph Explorer**: interactive code navigation with lazy loading
- **CLAUDE.md auto-generation** for AI agent context
- **8 language support**: Rust, Python, TypeScript, JavaScript, Go, Java, C/C++, Ruby
- **Multi-agent MCP config**: VS Code, Claude Code, Cursor, Codex, Antigravity IDE, Claude Desktop
- **Zero-configuration**: auto-detects workspace, auto-ingests on first open
- **Fully offline**: zero API keys, zero cloud, zero data egress

### Technical Details
- Rust backend with SQLite knowledge graph
- Tree-sitter AST parsing for 8 languages
- Query latency: <500ms p95 for all tools
- Auto-ingest runs as background process (non-blocking extension activation)
