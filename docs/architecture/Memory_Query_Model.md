# ARES Memory Query Model

## Overview
The Memory Query Model bridges the Retrieval Engine (which parses user intents and selects strategies) directly to the Repository Knowledge Graph. To avoid future rewrites of the retrieval layer, the Knowledge Graph is optimized around answering **first-class query categories**.

Instead of treating the graph as a flexible traversal sandbox, the graph explicitly exposes canonical questions as its primary API (`queries.rs`).

## Canonical Query Categories

Every question asked of the ARES Memory OS maps to one of the following primary intents. The Knowledge Graph directly implements graph traversal logic optimized for these categories.

### 1. Why (Justification & Origin)
**Query:** `why_does_this_exist()`
**Intent:** Traces a node (Code Artifact, Test, or Architecture) *up* the canonical hierarchy to find its originating Requirement and the Context/Problem that necessitated it.
**Paths:** `Code Artifact` $\rightarrow$ `Architecture` $\rightarrow$ `Decision` $\rightarrow$ `Requirement`

### 2. Who (Ownership & Approval)
**Query:** `who_owns_this()` and `what_approved_this()`
**Intent:** Identifies the human/team responsible for a node, and traces the approval governance layer.
**Paths:** `Decision` $\rightarrow$ `Owner` (via `ApprovedBy` and `OwnedBy` edges)

### 3. What (Evidence & Support)
**Query:** `what_evidence_supports_this()`
**Intent:** Validates a decision or architectural choice by retrieving the empirical data or external artifacts supporting it.
**Paths:** `Decision` $\rightarrow$ `Evidence` (via `SupportedBy` edge)

### 4. When & Evolution (History)
**Query:** `how_has_this_evolved()`
**Intent:** Examines the chronological progression and state changes of memory anchors.
**Paths:** `RequirementRevision` $\rightarrow$ `RequirementRevision` (via `Supersedes` or `DerivedFrom` edges)

### 5. Impact (Blast Radius & Risk)
**Query:** `what_breaks_if_changed()`
**Intent:** Performs a downward and lateral graph traversal to compute all downstream dependents and assess the severity (`ImpactRisk`) of modifying a source node.
**Paths:** `Architecture` $\rightarrow$ `Code Artifact` $\rightarrow$ `Test` (via `DependsOn`, `Validates` edges)

### 6. Debt (Knowledge Degradation)
**Query:** `what_knowledge_debt_exists()`
**Intent:** Scans the graph for anomalies, missing links, orphan nodes, and stale revisions to calculate the knowledge debt of a component.
**Paths:** Detects missing `TracesTo` or `ApprovedBy` edges on active memory anchors.

### 7. Gaps & Resolution (Health Execution)
**Query:** `what_gaps_exist()` and `how_do_we_fix_it()`
**Intent:** Explores the parallel Intelligence and Maintenance hierarchy to map identified failures to actionable plans.
**Paths:** `Gap` $\rightarrow$ `RootCause` $\rightarrow$ `Resolution`

## Integration with Retrieval Engine
The Retrieval Engine will map user input queries to one or more of these Canonical Query Categories. The `KnowledgeGraphStore` executes the corresponding canonical query method, which leverages a pre-optimized traversal path, drastically reducing the search space and avoiding generic $O(N)$ operations.
