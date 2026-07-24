# ARES Memory OS — Benchmarks

Comprehensive performance, token efficiency, and tool-call reduction benchmarks for ARES Memory OS across major open-source repositories.

---

## 1. Scale & Performance (Ingestion & Query Latency)

All benchmarks measured on Windows 11 / Release Build (`ares` v0.1.0).

| Repository | Files | Functions | AST Nodes | Ingest Time | Memory (MB) | Query Latency (p95) |
|------------|-------|-----------|-----------|-------------|-------------|----------------------|
| **tokio** | 848 | 24,192 | 86,400 | ~2m 10s | ~45 MB | < 15ms |
| **django** | 7,088 | 46,863 | 310,000 | ~9m 40s | ~120 MB | < 25ms |
| **react** | 7,006 | 38,200 | 280,000 | ~5m 15s | ~95 MB | < 20ms |
| **go** | 6,120 | 41,500 | 260,000 | ~6m 50s | ~110 MB | < 22ms |
| **ARES_Memory_os** | 1,619 | 7,199 | 45,200 | ~42s | ~35 MB | < 8ms |

---

## 2. Token Efficiency Comparison (Context Windows)

Comparing raw LLM file reading vs. ARES deterministic MCP responses.

| Query Scenario | Traditional File Read + Git (Tokens) | ARES MCP Response (Tokens) | Token Reduction |
|----------------|--------------------------------------|---------------------------|-----------------|
| **Why Exists** (`ares_why_exists`) | ~5,000 | ~420 | **91.6%** |
| **Who Owns** (`ares_who_owns`) | ~8,000 | ~310 | **96.1%** |
| **Impact Analysis** (`ares_impact`) | ~20,000 | ~650 | **96.8%** |
| **Architecture Overview** (`ares_architecture`) | ~35,000 | ~850 | **97.5%** |
| **Dead Code Detection** (`ares_dead_code`) | ~50,000 | ~280 | **99.4%** |
| **Overall Average** | **~23,600** | **~502** | **97.8%** |

---

## 3. Agent Tool-Call Reduction

| Workflow Task | Traditional Agent Steps | ARES Agent Steps | Reduction |
|---------------|-------------------------|------------------|-----------|
| Impact Analysis for Refactor | 11 calls (grep + 10 file reads) | 1 MCP call (`ares_impact`) | **90.9%** |
| Onboarding / Session Handoff | ~15 calls (reading docs/git log) | 1 MCP call (`ares_briefing`) | **93.3%** |
| Identifying Unused Code | ~25 calls (ast/lsp queries) | 1 MCP call (`ares_dead_code`) | **96.0%** |
| Traceability Verification | ~8 calls (searching commits/files) | 1 MCP call (`ares_traceability`) | **87.5%** |
| **Average Across Workflows** | **~14.7 calls** | **1 call** | **93.2%** |
