# Canonical Questions

ARES is a question-answering Memory Operating System. Instead of exposing "engines" or "crates" to the interface layer, we expose core canonical questions. These questions map directly to the `ares-query` services, acting as the singular interface standard for the CLI, MCP, API, and future UI.

## The Canonical Question Map

| Question                                 | Target Service                        | Evidence Produced                          | Deterministic |
|------------------------------------------|---------------------------------------|--------------------------------------------|---------------|
| Why does this exist?                     | `WhyQueryService`                     | Requirements, Decisions, Traceability      | Yes           |
| Where did this come from?                | `LineageQueryService`                 | Commits, PRs, Historical Snapshots         | Yes           |
| What breaks if changed?                  | `ImpactQueryService`                  | Dependency Edges, Coupling Metrics         | Yes           |
| Who owns it?                             | `OwnerQueryService`                   | Codeowners, Architectural Boundaries       | Yes           |
| What is the system health?               | `HealthQueryService`                  | Node counts, DB state, Error logs          | Yes           |
| What are the active capabilities?        | `CapabilityQueryService`              | Server Capability Registry                 | Yes           |
| What was inferred?                       | `BootstrapCandidateQueryService`      | Uncommitted candidate nodes                | No            |
| How much was covered by the bootstrap?   | `BootstrapCoverageQueryService`       | Total nodes vs inferred ratios             | Yes           |
| What gaps were closed?                   | `BootstrapGapClosureQueryService`     | Pre- vs Post-bootstrap gap deltas          | Yes           |
| Is it stale?                             | `LifecycleStatusQueryService`         | Last touched timestamps                    | Yes           |
| Can I trust it?                          | `LifecycleTrustQueryService`          | Decay rates, conflict anomaly scores       | Yes           |
| How fast is it decaying?                 | `LifecycleDecayQueryService`          | Variance over time, signal half-life       | Yes           |
| What needs revalidation?                 | `LifecycleRevalidationQueryService`   | Threshold evaluations on staleness         | Yes           |
| Is the repository structurally valid?    | `RepositoryValidationQueryService`    | Ontology rule checks, validation reports   | Yes           |

## Guiding Principle
When adding a new capability to ARES, do not ask "What API endpoint should I add for this engine?"
Instead ask, **"What question does this capability answer?"** 

If a new question is identified, it must be added to this canonical map.
