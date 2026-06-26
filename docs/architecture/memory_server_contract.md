# ARES Memory Server Contract

## Purpose
`ares-memory-server` is the **Gateway Layer** of the ARES Memory Operating System.

It has one job: expose the intelligence capabilities of ARES as stable, versioned HTTP contracts.

It contains **zero business logic**. All intelligence is delegated to the appropriate crate below.

---

## Service Inventory

| Route                  | Method | Owner Layer       | Delegate Crate               | Description                                  |
|------------------------|--------|-------------------|------------------------------|----------------------------------------------|
| `/health`              | GET    | Infrastructure    | Server-local                 | Liveness check for deployment monitoring     |
| `/version`             | GET    | Infrastructure    | Server-local                 | Returns server version and build context     |
| `/capabilities`        | GET    | Infrastructure    | Server-local registry        | Lists all registered ARES capabilities       |
| `/architecture`        | GET    | Infrastructure    | Server-local static          | Returns the validated ARES layering model    |
| `/repository`          | GET    | Orchestration     | `ares-query`                 | Returns repository identity and metadata     |
| `/repository/health`   | GET    | Orchestration     | `ares-query`                 | Returns aggregated repository health scores  |
| `/query/why`           | POST   | Orchestration     | `ares-query`                 | Explains why a node exists                   |
| `/query/lineage`       | POST   | Orchestration     | `ares-query`                 | Traces upstream/downstream lineage of a node |
| `/query/impact`        | POST   | Orchestration     | `ares-query`                 | Analyzes change impact risk for a node       |
| `/query/owner`         | POST   | Orchestration     | `ares-query`                 | Returns ownership information for a node     |

---

## Consumer Roles

| Consumer   | How it uses the server             |
|------------|------------------------------------|
| CLI        | `ares serve` → routes to API       |
| IDE Plugin | HTTP calls to query endpoints      |
| MCP Agents | Canonical tool definitions via API |
| CI/CD      | `/health` and `/architecture` checks |
| Enterprise | `/capabilities` for introspection  |

---

## Layering Rule

```
Core → Store → Retrieval → Intelligence → Orchestration (ares-query) → Gateway (ares-memory-server)
```

`ares-memory-server` may ONLY import:
- `ares-core` (for shared types)
- `ares-query` (canonical query surface)

It must NEVER import intelligence crates directly.

---

## DTO Policy

All HTTP response bodies use `QueryResult<T>` from `ares-query`.

No new DTOs are defined in `ares-memory-server` except those required for HTTP request bodies (e.g. `NodeQueryRequest`).

Engine-internal types never cross the HTTP boundary.
