# Repository Size Analysis

## Initial State Metrics (Pre-Cleanup)
- **`apps/`**: 276.17 MB (inflated by nested duplicate scaffold `apps/dashboard/apps/dashboard/node_modules`)
- **`scratch/`**: 244.15 MB (inflated by benchmark testing repositories like `nextjs-starter`)
- **Root Directory Files**: ~35 generated Markdown report/validation artifacts inflating Knowledge Graph ingestion size and causing recursive "reports about reports".

## Target State Improvements
- By removing `scratch/`, the repository base size shrinks by **~244 MB**.
- By removing the duplicate nested dashboard scaffold, the `apps/` directory shrinks substantially.
- By moving all validation artifacts to `reports/` and adding `reports/` to the ingestion exclusion list, the SQLite `ares.db` footprint and Graph Noise are drastically reduced.

## Graph Growth Implications
Generated validation artifacts and test-cloned repositories cause excessive node creation during `ares ingest`. Excluding them limits the graph strictly to Source Code, Tests, and explicit Knowledge Domains (`docs/requirements`, `docs/decisions`, `docs/architecture`, `docs/evidence`). This preserves deterministic retrieval quality and keeps cold ingest RSS footprint under 17.79 MB.
