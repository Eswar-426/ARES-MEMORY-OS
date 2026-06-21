# Repository Hygiene Audit

## 1. Nested Applications & Duplicate Folders
- **Identified**: `apps/dashboard/apps/dashboard`
- **Analysis**: The `apps/dashboard` directory contained a nested duplicate scaffold named `apps/dashboard` with a generic `package.json` for Vite+React. It lacked the actual dependencies (lucide-react, recharts, tailwindcss) and source code of the parent.
- **Resolution**: **Deleted**. The duplicate directory was an accidental scaffold error.

## 2. Abandoned & Generated Directories
- **Identified**: `scratch/`
- **Analysis**: The `scratch/` directory was generated during validation experiments (e.g. `nextjs-starter`) and contained over 244 MB of `node_modules` and test cases. Memory repositories must store knowledge, not generated validation artifacts.
- **Resolution**: **Deleted**. If benchmark clones are needed again, they can be regenerated on demand.

## 3. Knowledge Domain Standardization
- **Identified**: Loose root files (`*.md`)
- **Analysis**: The root repository contained over 35 loose validation, readiness, benchmark, and review markdown files. These files polluted the memory graph (generating reports about reports).
- **Resolution**: **Moved**. All non-core-knowledge markdown files were moved to `reports/validation/`, `reports/releases/`, `reports/audits/`, and `reports/performance/`.

## 4. Ingestion Exclusion Rules
- **Analysis**: Ingestion scanner previously processed generated reports and artifacts.
- **Resolution**: `reports`, `artifacts`, `scratch`, `node_modules`, `target`, `build`, `dist`, and `coverage` were added to the explicit ignore list in `crates/ares-ingestion/src/scanner.rs`.
