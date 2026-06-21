# ARES Repository Validation Matrix

To prove ARES works end-to-end, it must be validated against the following repository types.

## 1. Rust Ecosystem
| Repository | Expected Ingestion Result | Expected Graph Size | Expected MCP Checks | Expected Agent Workflows |
|---|---|---|---|---|
| **ARES** (Self-hosting) | Cargo workspace mapped, modules linked. | Large (>1000 nodes) | `ares_why_exists` on `MemoryFacade`, `ares_impact` on `ares_core`. | Agent generates test coverage report. |
| **ripgrep** | Single workspace, heavily optimized single binaries. | Medium | `ares_why_exists` on regex engine components. | Agent explains search pipeline architecture. |
| **cargo-watch** | Standard CLI tool. | Small | Compliance checks pass. | Agent simulates removing a CLI flag. |

## 2. TypeScript / Node Ecosystem
| Repository | Expected Ingestion Result | Expected Graph Size | Expected MCP Checks | Expected Agent Workflows |
|---|---|---|---|---|
| **Next.js App** | Pages/App router mapped, React components linked. | Medium | `ares_impact` on shared UI components. | Agent adds a new route honoring existing UI patterns. |
| **NestJS App** | Controllers, Providers, Modules mapped. DI graphed. | Medium | `ares_why_exists` on a specific Auth Guard. | Agent adds a new REST endpoint. |
| **Express App** | Routes and middleware mapped. | Small | Governance checks on route definitions. | Agent traces request lifecycle. |

## 3. Monorepo Ecosystem
| Repository | Expected Ingestion Result | Expected Graph Size | Expected MCP Checks | Expected Agent Workflows |
|---|---|---|---|---|
| **Turborepo** | Multiple packages ingested, inter-package dependencies mapped. | Large | `ares_impact` across package boundaries. | Agent updates a shared UI library and checks impact on apps. |
| **Nx Workspace** | Nx project graph correlated with ARES graph. | Large | `ares_coverage` across multiple libraries. | Agent explains the dependency graph between microservices. |
