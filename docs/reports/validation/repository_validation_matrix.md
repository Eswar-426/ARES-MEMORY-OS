# Repository Validation Matrix

This report summarizes the benchmark validation of ARES across both Tier A (Memory-Native) and Tier B (Standard External) repositories.

## Tier A: Memory-Native Repositories

These repositories are built with ARES traceability principles in mind.

| Repository | Ingestion Success | Latency (ms) | Peak RSS (MB) | Nodes | Edges | Gaps | Traceability Score |
|---|---|---|---|---|---|---|---|
| ARES | ✅ | 3901 | 24.57 | 2806 | 2753 | 1565 | 0.34% |
| Automyra | ✅ | 678 | 14.50 | 231 | 230 | 121 | 0.00% |

## Tier B: Standard External Repositories

These repositories are raw open-source projects. ARES evaluates ingestion performance, stability, and gap detection without artificial requirements.

| Repository | Ingestion Success | Latency (ms) | Peak RSS (MB) | Nodes | Edges | Gaps | Traceability Score |
|---|---|---|---|---|---|---|---|
| ripgrep | ✅ | 2117 | 14.89 | 521 | 520 | 302 | 0.00% (Expected Low) |
| cargo-watch | ✅ | 324 | 13.06 | 81 | 80 | 52 | 0.00% (Expected Low) |
| Next.js | ✅ | 106840 | 172.95 | 106080 | 132951 | 79620 | 10.30% (Expected Low) |
| NestJS | ✅ | 6396 | 27.82 | 5157 | 5662 | 3106 | 15.45% (Expected Low) |
| Turborepo | ✅ | 44273 | 54.71 | 19770 | 26000 | 13940 | 2.64% (Expected Low) |
| Nx Workspace | ✅ | 39452 | 58.59 | 22774 | 38875 | 12955 | 14.72% (Expected Low) |

## Analysis
- **Stability**: 100% Ingestion Success. No Panics. No Crashes.
- **Performance**: Large repositories (Next.js with 106K nodes) were ingested within ~106 seconds, utilizing 172.95 MB peak RSS.
- **Memory Constraints**: The architecture remains lightweight, successfully keeping Peak RSS completely bounded under 200MB even for massive web frameworks.
