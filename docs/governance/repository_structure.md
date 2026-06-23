# Repository Structure Governance

ARES Memory OS enforces a strict four-domain repository architecture. This guarantees predictability, prevents root-directory pollution, and clearly delineates machine-generated data from human documentation and source code.

## The Four Primary Domains

1. **`crates/` (Source Code)**
   - Contains all Rust library and binary crates.
   - Strictly reserved for human-authored source code and tests.
   - Do not place any generated runtime data or temporary logs here.

2. **`docs/` (Human Documentation)**
   - The single source of truth for architectural planning, human-readable reports, and governance.
   - Subdirectories:
     - `/architecture/`: Strategic roadmaps, overviews, hierarchy models.
     - `/governance/`: Standards, repository structure, and release policies.
     - `/reports/`: Evaluated human-readable reports for health, validation, and certifications.
     - `/decisions/`: ADRs (Architecture Decision Records).
     - `/deployment/`: Instructions for releasing and hosting the OS.
     - `/user-guides/`: Manuals for end-users.

3. **`artifacts/` (Generated Outputs)**
   - Houses machine-generated assets meant for sharing, inspecting, or exporting.
   - These are NOT human documentation.
   - Subdirectories:
     - `/openapi/`: Auto-generated API definitions (e.g., `openapi.json`).
     - `/validation_runs/`: CI/CD outputs, temporary debug dumps (e.g., `out1.txt`), tree scans.
     - `/exports/`: Exported SARIF reports or external integration files.
     - `/snapshots/`: Checkpointed system configurations or historical `base.json` states.
     - `/benchmarks/`: Output data from benchmark runs.

4. **`.ares/` (Runtime Data)**
   - The operational footprint of the Memory Server.
   - This directory may be added to `.gitignore` depending on the environment, except for explicit configuration files.
   - Subdirectories/Files:
     - `memory.db`: The core graph database.
     - `build_manifest.json`: The incremental pipeline log.
     - `/cache/`: Temporary processing cache for intelligence engines.
     - `/logs/`: Application execution logs.
     - `/temp/`: Ephemeral state.

## Enforcing Hygiene
- **Never** place `*.md` reports in the repository root.
- **Never** save `.db` files or `.json` artifacts directly in the root.
- Any temporary script (`debug.py`, `test_parser.rs`) should immediately be placed in `scripts/debug/`.
