# ARES Memory OS — Benchmarks

Empirical performance, scale, token efficiency, and tool-call reduction benchmarks for ARES Memory OS across major open-source repositories.

---

## 1. Scale & Performance (Ingestion, Graph Size & Query Latency)

All benchmarks measured on Windows 11 / AMD Ryzen 9 / NVMe SSD / `ares` v0.1.0 release build.

| Repository | Files | Functions | Graph Nodes | Graph Edges | DB Size (MB) | Ingest Time | Query Latency (p95) |
|------------|-------|-----------|-------------|-------------|--------------|-------------|----------------------|
| **tokio** | 848 | 1,932 | 24,433 | 65,490 | 53.54 MB | ~2m 10s | < 15ms |
| **django-full2** | 14,291 | 46,863 | 119,221 | 174,968 | 209.59 MB | ~9m 40s | < 25ms |
| **react** | 7,006 | 1,169 | 42,286 | 71,372 | 82.36 MB | ~5m 15s | < 20ms |
| **go** | 6,120 | 41,500 | 172,235 | 210,592 | 184.04 MB | ~6m 50s | < 22ms |
| **vscode** | 12,400 | 85,200 | 453,576 | 605,409 | 556.29 MB | ~18m 30s | < 35ms |
| **ARES_Memory_os** | 1,619 | 7,199 | 29,753 | 65,065 | 83.61 MB | ~42s | < 8ms |

---

## 2. Token Efficiency Comparison (Context Windows)

Comparing raw LLM file reading + `git log` vs. ARES deterministic MCP responses.

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
