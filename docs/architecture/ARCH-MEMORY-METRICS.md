# ARES MemoryOS Governance Metrics

This document formalizes the definitions, formulas, and thresholds for ARES MemoryOS Governance Metrics.
These metrics are the foundation of the Memory Gatekeeper and determine whether a Pull Request is allowed to merge.

> [!WARNING]
> **Metric Definition Changes**
> Any modifications to the definitions, formulas, or thresholds in this document require:
> 1. An Architecture Decision Record (ADR)
> 2. A Migration Plan
> 3. A Certification Re-run (`certify_synthetic_matrix.ps1` and `certify_real_matrix.ps1`)
> 4. A Version Bump

---

## 1. Memory Coverage

**Version:** 1.0

**Purpose:** 
Measure the proportion of the repository's functional footprint that has attached architectural memory (ownership and traceability). It answers the question: "How much of this codebase is understood by ARES?"

**Formula:**
`Memory Coverage = (Nodes with Owners + Nodes with Traceability) / Total Eligible Nodes`
*Note: A perfectly covered node contributes maximally. A node with an owner but no traceability contributes partially.*

**Inputs:**
- Knowledge Graph Nodes (filtered to `Code`, `Configuration`, `Infrastructure` categories).
- Edges (`OwnedBy`, `Implements`, `Satisfies`, `Contains`).

**Thresholds:**
- Target: 100%
- Warning: < 80%

**SoftFail Rules:**
- Coverage drops by > 3% compared to the baseline.

**HardFail Rules:**
- Coverage drops by > 5% compared to the baseline.
- An explicitly assigned Owner is removed without a replacement.

**Examples:**
- Adding 10 new Rust files without a `.github/CODEOWNERS` entry drops Memory Coverage.
- Adding a new microservice without updating the ADRs drops Memory Coverage.

**Known Blind Spots:**
- Generated code artifacts that are committed to the repository.
- Third-party vendor code included directly in the source tree.

---

## 2. Memory Debt

**Version:** 1.0

**Purpose:**
Quantify the accumulated architectural violations, missing context, and orphaned logic. It measures the liability of undocumented or disconnected code.

**Formula:**
`Memory Debt = Σ (Violation Severity Weight)`
Weights: `Critical (5)`, `High (3)`, `Medium (2)`, `Low (1)`

**Inputs:**
- Output from the `CoverageEngine` and `TraceabilityEngine`.
- Specifically: Orphaned decisions, code without owners, requirements without code.

**Thresholds:**
- Target: 0
- Warning: > 10% of total nodes

**SoftFail Rules:**
- Debt increases by > 5% compared to the baseline.

**HardFail Rules:**
- Debt increases by > 25% compared to the baseline.
- A `Critical` debt violation is introduced (e.g., modifying a core system without updating the corresponding security ADR).

**Examples:**
- Deleting an ADR but leaving the code that implemented it causes the code to become orphaned, increasing Debt.
- Adding a requirement without writing the code increases Debt.

**Known Blind Spots:**
- Stale documentation (an ADR exists, but its content is completely outdated and no longer matches the code).

---

## 3. Memory Health

**Version:** 1.0

**Purpose:**
Provide an aggregate, normalized score of the overall state of the repository's memory. It acts as the primary executive indicator.

**Formula:**
`Memory Health = (Coverage Score * 0.4) + ((1 - Normalized Debt) * 0.4) + (Traceability Density * 0.2)`
*(Scaled to 0-100)*

**Inputs:**
- Memory Coverage
- Memory Debt (Normalized against total repository size)
- Traceability Density

**Thresholds:**
- Target: 90+ (Healthy)
- Warning: < 70 (At Risk)
- Critical: < 50 (Unhealthy)

**SoftFail Rules:**
- Health regresses by > 5% compared to the baseline.

**HardFail Rules:**
- Health regresses by > 10% compared to the baseline.
- Overall Health drops below 50.

**Examples:**
- A large refactor that deletes traceability links and adds undocumented files will severely impact Memory Health.

**Known Blind Spots:**
- Very small repositories can experience massive health swings from single file changes.

---

## 4. Memory Drift

**Version:** 1.0

**Purpose:**
Detect when the codebase execution or structure diverges from the explicit architectural constraints defined in the memory graph.

**Formula:**
`Memory Drift = (Deviating Constraints / Total Constraints)`

**Inputs:**
- Constraint nodes in the graph.
- Runtime signals (future) or static analysis checks that map to constraints.

**Thresholds:**
- Target: 0%
- Warning: > 0%

**SoftFail Rules:**
- A `Low` or `Medium` severity constraint is violated.

**HardFail Rules:**
- A `High` or `Critical` severity constraint is violated (e.g., a dependency rule is broken).

**Examples:**
- An ADR specifies that `Module A` must not depend on `Module B`. If `import module_b` is found in `Module A`, drift is detected.

**Known Blind Spots:**
- Implicit constraints that are not explicitly modeled in the graph as Constraint nodes.
- Semantic drift (the code follows the constraints mechanically, but violates the spirit of the architecture).

---

## 5. Traceability Density

**Version:** 1.0

**Purpose:**
Measure the depth and interconnectedness of the knowledge graph. A repository with high traceability density has clear paths from Requirements to Decisions to Code.

**Formula:**
`Traceability Density = Total Traceability Edges / Total Eligible Nodes`

**Inputs:**
- Number of `Implements`, `Satisfies`, `Validates` edges.
- Total count of `Requirement`, `Decision`, and `Code` nodes.

**Thresholds:**
- Target: > 1.5 (On average, every node is connected to at least 1.5 other semantic nodes).
- Warning: < 0.5

**SoftFail Rules:**
- N/A (Traceability Density is primarily an analytical metric, not a direct gating metric, though it influences Health).

**HardFail Rules:**
- N/A

**Examples:**
- A codebase where every PR links to an Issue, every Issue links to a Requirement, and every Requirement links to an ADR has very high Traceability Density.

**Known Blind Spots:**
- Spurious links (e.g., developers linking everything to a single "misc" requirement just to satisfy checks).
