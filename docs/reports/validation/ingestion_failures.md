# Ingestion Failures

## Automated Validation Log

| Repository | Failure Encountered | Severity | Description |
| :--- | :--- | :--- | :--- |
| **cargo-watch** | None | - | Cleanly parsed traits and structs. |
| **ripgrep** | None | - | Cleanly parsed complex Rust macros and traits. |
| **express** | None | - | Core CommonJS/JS exports handled effectively. |
| **nestjs-starter** | None | - | Decorator-heavy classes successfully parsed without panicking. |
| **nextjs-starter** | None | - | Functional components and React specifics parsed without panics. |
| **turborepo-basic** | None | - | Monorepo structure traversed correctly. |
| **nx-workspace** | None | - | Deep angular/nx nested workspaces traversed correctly. |

## Panics

* **Panics during ingestion:** 0
* **Crash to Desktop:** 0
* **OOM (Out of Memory):** 0

*The removal of raw `.unwrap()` calls during Sprint 1 completely stabilized the ingestion process across 7 wildly divergent architectures.*
