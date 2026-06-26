# ARES Query Contracts

This document formalizes the public DTO (Data Transfer Object) boundary of the ARES Query layer (`ares-query`). 

## The Prime Directive
**All internal intelligence models must be coerced into `QueryResult<T>` before leaving `ares-query`.**

Internal engine models (e.g., `LifecycleState`, `BootstrapMatrix`) must **never** be exposed via HTTP, MCP, or CLI directly. They must be wrapped with verifiable evidence and confidence scores.

## The `QueryResult<T>` Contract
```rust
pub struct QueryResult<T> {
    pub data: T,
    pub evidence: QueryEvidence,
    pub metadata: QueryMetadata,
}

pub struct QueryEvidence {
    pub node_ids: Vec<String>,
}

pub struct QueryMetadata {
    pub confidence: f64,
    pub generated_at: DateTime<Utc>,
    pub repository_id: String,
}
```

## Contract Specifications

### 1. WhyQueryService
* **Question**: Why does this node exist?
* **Inputs**: `project_id: ProjectId`, `node_id: String`
* **Output Type**: `WhyResponse`
* **Evidence Source**: Requirement nodes, Decision nodes, Traceability edges
* **Confidence Source**: Edge traversal depth and validity
* **Deterministic**: Yes

### 2. LineageQueryService
* **Question**: Where did this node come from?
* **Inputs**: `project_id: ProjectId`, `node_id: String`
* **Output Type**: `LineageResponse`
* **Evidence Source**: Commits, PRs, historical graph states
* **Confidence Source**: Continuity of version history
* **Deterministic**: Yes

### 3. ImpactQueryService
* **Question**: What breaks if this node is changed?
* **Inputs**: `project_id: ProjectId`, `node_id: String`
* **Output Type**: `ImpactResponse`
* **Evidence Source**: Downstream dependency edges
* **Confidence Source**: Static analysis coupling metrics
* **Deterministic**: Yes

### 4. OwnerQueryService
* **Question**: Who owns this node?
* **Inputs**: `project_id: ProjectId`, `node_id: String`
* **Output Type**: `OwnerResponse`
* **Evidence Source**: Codeowners, PR authors, architectural domains
* **Confidence Source**: Ownership rule matches
* **Deterministic**: Yes

### 5. BootstrapCandidateQueryService
* **Question**: What was inferred?
* **Inputs**: `project_id: ProjectId`
* **Output Type**: `BootstrapCandidateResponse`
* **Evidence Source**: Candidate nodes
* **Confidence Source**: Heuristic match strength
* **Deterministic**: No (Heuristic-based)

### 6. BootstrapCoverageQueryService
* **Question**: How much was covered by the bootstrap?
* **Inputs**: `project_id: ProjectId`
* **Output Type**: `BootstrapCoverageResponse`
* **Evidence Source**: Total nodes vs. inferred nodes
* **Confidence Source**: N/A (Statistical)
* **Deterministic**: Yes

### 7. BootstrapGapClosureQueryService
* **Question**: What gaps were closed?
* **Inputs**: `project_id: ProjectId`
* **Output Type**: `BootstrapGapClosureResponse`
* **Evidence Source**: Pre-bootstrap vs post-bootstrap gap metrics
* **Confidence Source**: Gap closure verification rules
* **Deterministic**: Yes

### 8. LifecycleStatusQueryService
* **Question**: Is it stale?
* **Inputs**: `project_id: ProjectId`, `node_id: String`
* **Output Type**: `LifecycleStatusResponse`
* **Evidence Source**: Last touched timestamp
* **Confidence Source**: N/A
* **Deterministic**: Yes

### 9. LifecycleTrustQueryService
* **Question**: Can I trust it?
* **Inputs**: `project_id: ProjectId`, `node_id: String`
* **Output Type**: `LifecycleTrustResponse`
* **Evidence Source**: Decay metrics, anomaly reports
* **Confidence Source**: Aggregated trust score algorithms
* **Deterministic**: Yes

### 10. LifecycleDecayQueryService
* **Question**: How fast is it decaying?
* **Inputs**: `project_id: ProjectId`, `node_id: String`
* **Output Type**: `LifecycleDecayResponse`
* **Evidence Source**: Temporal analysis of usage/updates
* **Confidence Source**: Statistical variance
* **Deterministic**: Yes

### 11. LifecycleRevalidationQueryService
* **Question**: What needs revalidation?
* **Inputs**: `project_id: ProjectId`, `node_id: String`
* **Output Type**: `LifecycleRevalidationResponse`
* **Evidence Source**: Staleness thresholds crossed
* **Confidence Source**: Rule evaluation
* **Deterministic**: Yes

### 12. RepositoryValidationQueryService
* **Question**: Is the repository structurally valid?
* **Inputs**: `project_id: ProjectId`
* **Output Type**: `RepositoryValidationResponse`
* **Evidence Source**: Reality validation reports, ontological checks
* **Confidence Source**: Rule coverage
* **Deterministic**: Yes
