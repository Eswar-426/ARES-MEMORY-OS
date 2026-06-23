# Repository Structure Governance

ARES Memory OS enforces a strict architectural taxonomy for its repository. This guarantees predictability, prevents root-directory entropy, and cleanly separates machine-generated data from human documentation and source code. 

**This file is authoritative. All future phases and pull requests must comply with these domain boundaries.**

## The Domain Rules

```text
Runtime files      -> .ares/
Generated assets   -> artifacts/
Documentation      -> docs/
Source code        -> crates/
Applications       -> apps/
Extensions         -> extensions/
Datasets           -> datasets/
Scripts            -> scripts/
Configurations     -> configs/
```

## Detailed Domain Breakdown

1. **`crates/` (Source Code)**
   - Contains all core Rust library and binary crates.
   - Strictly reserved for human-authored source code and tests.
   - Do not place any generated runtime data or temporary logs here.

2. **`docs/` (Human Documentation)**
   - The single source of truth for architectural planning, human-readable reports, and governance.
   - Subdirectories:
     - `/architecture/`: Strategic roadmaps, overviews, hierarchy models.
     - `/governance/`: Standards, repository structure, and release policies.
     - `/reports/`: Evaluated human-readable reports for health, validation, and certifications.
     - `/decisions/`: ADRs (Architecture Decision Records) preserving the institutional memory of ARES.
     - `/releases/`: Certified release records and phase checklists.
     - `/deployment/`: Instructions for releasing and hosting the OS.
     - `/user-guides/`: Manuals for end-users.

3. **`artifacts/` (Generated Assets)**
   - Houses machine-generated assets meant for sharing, inspecting, or exporting.
   - These are strictly non-human documentation.
   - Subdirectories:
     - `/openapi/`: Auto-generated API definitions (e.g., `openapi.json`).
     - `/validation_runs/`: CI/CD outputs, temporary debug dumps (e.g., `out1.txt`), tree scans.
     - `/exports/`: Exported SARIF reports or external integration files.
     - `/snapshots/`: Checkpointed system configurations or historical `base.json` states.
     - `/benchmarks/`: Output data from benchmark runs.
     - `/generated/`: Any other automatically compiled machine outputs.

4. **`.ares/` (Runtime Data)**
   - The operational footprint of the Memory Server.
   - Subdirectories/Files:
     - `memory.db`: The core graph database.
     - `build_manifest.json`: The incremental pipeline log.
     - `/cache/`: Temporary processing cache for intelligence engines.
     - `/logs/`: Application execution logs.
     - `/temp/`: Ephemeral state.

5. **`configs/` (Configurations)**
   - Authoritative YAML/TOML specifications controlling global behavior.
   - e.g., `models.yaml`, `providers.yaml`, `memory.yaml`.

6. **`apps/`, `extensions/`, `datasets/`, `scripts/`**
   - High-level containers for product surface area, IDE tooling, evaluation sets, and maintenance operations respectively.

## Enforcing Hygiene
- **Never** place `*.md` reports in the repository root (except `README.md` and `CHANGELOG.md`).
- **Never** save `.db` files or `.json` artifacts directly in the root.
- Any temporary debug script must immediately be placed in `scripts/debug/`.
