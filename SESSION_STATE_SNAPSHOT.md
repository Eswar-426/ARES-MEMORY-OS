
# ARES Memory OS — Session State Snapshot
# Date: 2026-07-14 19.37
# Status: All backend working, UI needs clean HTML rewrite

## WHAT WORKS (DO NOT TOUCH THESE)
- All Rust tools: briefing, health_check, dead_code, context_file, hotspots, co_change, decay
- All MCP registrations in main.rs
- Binary bundling (BUG-005 fixed)
- State machine (state.ts, setState in extension.ts)
- Watcher with logging + READY guard
- Auto-ingest (ingest not scan)
- Who Owns + Decisions commands registered
- Query type inference in parseAresResponse
- renderBriefing, renderDeadCode, renderWhoOwns, renderGaps, renderDashboard functions
- renderMarkdown with double-escaped regexes
- renderQueryTypeBadgeHtml
- Dynamic panel titles
- hideAllSections function

## WHAT NEEDS CLEAN REWRITE (NOT PATCHES)
- The HTML template in queryPanel.ts has dead sections that should be REMOVED not hidden
- The generic renderer (why_exists, impact, drift) needs premium styling to match full-page views
- renderAnswer needs to work inside a clean HTML structure, not the old section-based layout

## KEY LESSONS LEARNED
1. NEVER hide DOM elements — remove unused ones from HTML
2. NEVER use backtick regex inside template literals — use new RegExp()
3. NEVER use git restore on directories
4. NEVER use Python regex on TypeScript files
5. ALWAYS double-escape backslashes in template literal regexes
6. ALWAYS rebuild VSIX after TypeScript changes (npm run compile alone is not enough)
7. ALWAYS verify MCP uses bundled binary (check for 'bundled' in startup log)
8. context_file.rs needs project_id parameter, not path parsing
9. hotspots.rs: touches edges go commit→file, so join on to_node_id
10. hotspots.rs: created_at is microseconds, not date strings
11. All graph edge queries need 'valid_until IS NULL' filter

## FILE LOCATIONS
- Rust: crates/ares-intelligence/src/{briefing,context_file,dead_code,hotspots,co_change,decay}.rs
- Rust: crates/ares-mcp/src/main.rs (all tool handlers)
- TS: extensions/ares-memory-vscode/src/{extension,state,watcher,queryPanel,mcp-client,commands/query,commands/health,commands/dashboard,commands/graph,commands/cli,binaryDownloader}.ts
- Build: package.ps1 (root), BENCHMARKS.md (root)

## DATABASE SCHEMA NOTES
- Timestamps in MICROSECONDS (* 1_000_000)
- Node IDs are strings
- Edge: from_node_id → to_node_id
- touches: commit → file (from=commit, to=file)
- No 'metadata' column — it's 'properties'
- No 'valid_until IS NULL' = dead edges returned
- project_id comes from session, not Path::file_name()

