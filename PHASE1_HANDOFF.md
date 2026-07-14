# ARES Memory OS — Session Handoff Document
# Created: 2026-07-13
# Purpose: Complete state snapshot for continuing in a new chat session

=================================================================
CURRENT STATE: WHAT IS WORKING
=================================================================

BACKEND (Rust) — ALL COMPILED AND PACKAGED
- [x] ares_briefing — Returns real data (1868 files, health 49/100)
- [x] ares_health_check — Returns real data with hotspots, decay gaps
- [x] ares_dead_code — Returns valid JSON structure
- [x] ares_generate_context_file — Writes .ares/CLAUDE.md
- [x] ares_architecture — Returns data with hidden_coupling
- [x] ares_decisions — Returns with provenance badges, human-first sort
- [x] ares_why_exists — Returns with decay enrichment on decisions
- [x] Hotspot detection (hotspots.rs) — Integrated into health check
- [x] Co-change detection (co_change.rs) — Integrated into architecture
- [x] Decision confidence decay (decay.rs) — Integrated into 3 handlers
- [x] Agent provenance tracking — In record_decision and annotate
- [x] Language parsers: C#, PHP (working), Kotlin (graceful skip, query disabled)
- [x] Extension state machine (state.ts) — CHECKING/READY/INGESTING/DISMISSED
- [x] File watcher — Expanded glob, pauses during INGESTING state
- [x] context_file.rs — Rewritten with SqliteGapRepository, proper edge chains
- [x] briefing.rs — Fixed primary language, bidirectional edges, shared health score
- [x] Safe tree-sitter queries (try_build_query in mod.rs)
- [x] BENCHMARKS.md generated (97% token reduction, 88% tool call reduction)

EXTENSION (TypeScript) — CURRENTLY PACKAGED IN VSIX
- [x] All original commands working (health, why_exists, impact, who_owns, timeline, drift, compare, simulate, search, decisions, architecture, traceability, coverage)
- [x] ares.briefing command registered
- [x] ares.findDeadCode command registered  
- [x] ares.generateContextFile command registered
- [x] parseAresResponse briefing data mapping (result.project -> flat)
- [x] renderBriefing function (full dashboard UI)
- [x] Dispatch: query_type === "briefing" routes to renderBriefing
- [x] Provenance/staleness CSS badges in queryPanel.ts
- [x] buildDecisions with provenance badges
- [x] Activation events cleaned (onStartupFinished, onWorkspaceContains:.ares)
- [x] State machine in extension.ts (consent notification, no blocking ingest)

=================================================================
PRECISE AUDIT: WHAT git restore DESTROYED (verified 2026-07-13 15:50)
=================================================================

The following items existed in the working session but were reverted
by "git restore extensions/ares-memory-vscode/src/" and have NOT
been rebuilt yet:

[LOST-001] renderDeadCode function in queryPanel.ts
  - PREVIOUSLY: Rendered dead files/functions in grouped scrollable list
  - CURRENTLY: Falls back to plain markdown text renderer
  - NEED: Custom render function + dispatch in showData

[LOST-002] Dead code dispatch in showData
  - NEED: if (data.query_type === "dead_code") { renderDeadCode(data); return; }

[LOST-003] Provenance/staleness CSS classes in queryPanel.ts
  - ALL of these were wiped:
    .provenance-badge, .provenance-human, .provenance-agent
    .staleness-fresh, .staleness-aging, .staleness-stale, .staleness-expired
  - These are needed by buildDecisions to show human/agent badges
  - NOTE: buildDecisions function itself still exists and references these classes

[LOST-004] Watcher state guard in watcher.ts
  - PREVIOUSLY: if (getState() !== AresState.READY) { skip flush }
  - CURRENTLY: Watcher runs even during INGESTING state
  - NEED: Add import of getState/AresState, add guard in flushQueue

[LOST-005] Watcher expanded glob in watcher.ts
  - PREVIOUSLY: **/*.{rs,ts,tsx,js,jsx,md,toml,json,py,go,java,c,cpp,h,hpp,cc,cxx,rb,cs,php,kt,kts}
  - CURRENTLY: **/*.{rs,ts,tsx,js,jsx,md,toml,json} (missing 10 languages)
  - NEED: Replace the glob string

[LOST-006] Duplicate "Most active" in Briefing
  - Rust backend recent_summary no longer includes "Most active:" (fixed)
  - VERIFY: TypeScript side should be the only one showing it

=================================================================
KNOWN BUGS TO FIX BEFORE PHASE 2
=================================================================

[BUG-001] CLAUDE.md shows "unknown" project name
  - Root cause: MCP tool handler passes workspace path incorrectly
  - Location: crates/ares-mcp/src/main.rs context_file_tool handler
  - The handler calls generate_context_file(&store, &pp, None)
  - But pp might be empty string or "." instead of full workspace path
  - FIX: Verify project_path is resolved correctly in main.rs

[BUG-002] CLAUDE.md entry points still show 0 inbound
  - The rewritten context_file.rs should fix this
  - BUT: The running MCP server may be using old cached binary
  - FIX: Must kill old ares-mcp.exe process before testing
  - VERIFY: After fresh window reload, check .ares/CLAUDE.md content

[BUG-003] CLAUDE.md health score mismatch (90 vs 49)
  - Same root cause as BUG-001 — old binary running
  - The rewritten context_file.rs uses SqliteGapRepository
  - FIX: Same as BUG-001

[BUG-004] Dead code "no memory" error
  - MCP returns empty/null when no dead code found
  - The webview shows "ARES does not have memory for that"
  - This is actually CORRECT behavior for a repo with no dead code
  - But the empty state message should be more helpful
  - FIX: Add dead_code case to showData empty state handling

[BUG-005] Auto-downloader pulls old binaries
  - When extension installs from VSIX, ensureBinaries downloads from GitHub
  - This overwrites the locally compiled binaries
  - FIX: Either disable auto-download for local VSIX installs, or
         ensure the VSIX includes the correct binaries in binaries/windows/

=================================================================
PHASE 1 TESTING CHECKLIST (NOT YET EXECUTED)
=================================================================

MANUAL VERIFICATION NEEDED:
[ ] Install VSIX, reload window, verify MCP uses LOCAL binaries (not auto-downloaded)
[ ] ARES: Briefing → shows dashboard with real data (not plain text)
[ ] ARES: Briefing → health score matches Health Check (both 49)
[ ] ARES: Briefing → no duplicate "Most active" line
[ ] ARES: Briefing → freshness badge shows green (< 1 hour)
[ ] ARES: Health Check → shows gaps, hotspots, decay
[ ] ARES: Find Dead Code → shows proper UI (not plain text) or helpful empty state
[ ] ARES: Generate Context File → writes .ares/CLAUDE.md with:
    [ ] Project name = ARES_Memory_os (not "unknown")
    [ ] Health score = 49 (not 90)
    [ ] Entry points with real inbound counts (not all 0)
    [ ] Critical files are source code (not Cargo.toml)
    [ ] Ownership shows real contributors
    [ ] Tech stack has mapped names only (no raw extensions like "css", "png")
[ ] ARES: Why Exists on a real file → returns creation commit
[ ] ARES: Impact on a real file → returns dependents
[ ] ARES: Who Owns on a real file → returns contributor percentages
[ ] ARES: Decisions → shows provenance badges (human/agent)
[ ] File watcher → modifies file, verify no crash during READY state
[ ] State machine → fresh workspace shows consent notification

DEFERRED TO PHASE 2:
[ ] Test on 5 external repos (tokio, django, react, go, vscode)
[ ] C# parser on real .cs file
[ ] PHP parser on real .php file
[ ] Kotlin parser graceful skip on Kotlin repo
[ ] Incremental ingest verification

=================================================================
FILE LOCATIONS REFERENCE
=================================================================

RUST BACKEND:
- crates/ares-intelligence/src/briefing.rs — Briefing generation
- crates/ares-intelligence/src/context_file.rs — CLAUDE.md generation
- crates/ares-intelligence/src/dead_code.rs — Dead code detection
- crates/ares-intelligence/src/hotspots.rs — Hotspot calculation
- crates/ares-intelligence/src/co_change.rs — Hidden coupling detection
- crates/ares-intelligence/src/decay.rs — Decision confidence decay
- crates/ares-intelligence/src/state.ts — (does not exist, state is in TS)
- crates/ares-scanner/src/languages/mod.rs — try_build_query safe wrapper
- crates/ares-scanner/src/languages/csharp.rs — C# parser
- crates/ares-scanner/src/languages/php.rs — PHP parser
- crates/ares-scanner/src/languages/kotlin.rs — Kotlin parser (query disabled)
- crates/ares-mcp/src/main.rs — All MCP tool registrations

TYPESCRIPT EXTENSION:
- extensions/ares-memory-vscode/src/extension.ts — Activation lifecycle
- extensions/ares-memory-vscode/src/state.ts — AresState enum
- extensions/ares-memory-vscode/src/watcher.ts — File change watcher
- extensions/ares-memory-vscode/src/commands/query.ts — Tool commands + parseAresResponse
- extensions/ares-memory-vscode/src/queryPanel.ts — Webview rendering
- extensions/ares-memory-vscode/src/mcp-client.ts — MCP connection
- extensions/ares-memory-vscode/package.json — Command declarations

SCRIPTS:
- scripts/run_benchmarks.py — Benchmark suite
- scripts/package.ps1 — Build and package VSIX
- PHASE1_TEST_REPORT.md — Test checklist
- BENCHMARKS.md — Performance results

=================================================================
KEY PATTERNS TO FOLLOW
=================================================================

1. NEVER use Python regex scripts to modify TypeScript files
   - They corrupt escape sequences (\n becomes literal newline)
   - Use VS Code native editing or single-line targeted Python inserts

2. NEVER use git restore on the entire src/ directory
   - It reverts ALL uncommitted work, not just the broken change
   - Restore individual files only: git restore path/to/file.ts

3. ALWAYS verify MCP uses LOCAL binaries after VSIX install
   - Check startup log for (Auto-Downloaded) vs local path
   - If auto-downloaded, the new Rust code is NOT being used

4. ALWAYS rebuild after Rust changes
   - cargo check → package.ps1 → install VSIX → reload window
   - Old ares-mcp.exe process persists across reloads

5. For queryPanel.ts additions:
   - Add dispatch in showData() first
   - Add render function before renderHeader()
   - npm run compile to verify
   - Then package

=================================================================
NEXT STEPS (IN ORDER)
=================================================================

1. Fix renderDeadCode in queryPanel.ts (LOST-001, LOST-002)
2. Verify BUG-001 through BUG-003 by testing CLAUDE.md output
3. Fix BUG-005 (auto-downloader) if needed
4. Complete all Phase 1 manual verifications above
5. Only then: proceed to Phase 2 (5 external repos)
