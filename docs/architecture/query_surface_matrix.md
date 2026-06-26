# ARES Query Surface Matrix

ARES is ultimately a question-answering platform. This matrix defines how core domain questions map directly to the `ares-query` services that answer them. 

## The Matrix

| Domain                     | Question                               | Service                               |
|----------------------------|----------------------------------------|---------------------------------------|
| **Core Intelligence**      | Why does this exist?                   | `WhyQueryService`                     |
|                            | Where did this come from?              | `LineageQueryService`                 |
|                            | What breaks if changed?                | `ImpactQueryService`                  |
|                            | Who owns it?                           | `OwnerQueryService`                   |
|                            | What is the system health?             | `HealthQueryService`                  |
|                            | What are the active capabilities?      | `CapabilityQueryService`              |
| **Bootstrap Intelligence** | What was inferred?                     | `BootstrapCandidateQueryService`      |
|                            | How much was covered?                  | `BootstrapCoverageQueryService`       |
|                            | What gaps were closed?                 | `BootstrapGapClosureQueryService`     |
| **Lifecycle Intelligence** | Is it stale?                           | `LifecycleStatusQueryService`         |
|                            | Can I trust it?                        | `LifecycleTrustQueryService`          |
|                            | How fast is it decaying?               | `LifecycleDecayQueryService`          |
|                            | What needs revalidation?               | `LifecycleRevalidationQueryService`   |
| **Repository Validation**  | Is the repository structurally valid?  | `RepositoryValidationQueryService`    |
