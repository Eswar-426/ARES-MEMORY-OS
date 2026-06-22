# ARES Memory Hierarchy

This document serves as the constitutional baseline for memory relationships within ARES. It defines the strict layers, allowed dependencies, and promotion workflows that govern the ARES Knowledge Graph.

## Core Memory Hierarchy

The authoritative sequence of engineering causality in ARES strictly follows:

```
Requirements
    ↓
Decisions
    ↓
Architecture
    ↓
Code
    ↓
Tests
    ↓
Runtime Signals
    ↓
Outcomes
```

## Relationship Rules

To prevent semantic pollution and unexplainable lineage, ARES enforces strict relational boundaries.

### Allowed Relationships
The following directional relationships are structurally valid:
- `Requirement -> Decision`: A business need motivates a technical choice.
- `Decision -> Architecture`: A choice manifests as a structural component.
- `Architecture -> Code`: A component is implemented by source files.
- `Code -> Test`: Source code is validated by test suites.
- `Test -> Runtime Signal`: Tests influence or match runtime execution behavior.
- `Runtime Signal -> Outcome`: System execution leads to measurable outcomes.

### Forbidden Relationships
Direct linkage bypassing the hierarchy degrades reasoning capability. The following relationships are strictly blocked:
- `Code -> Requirement`: Code cannot independently generate a requirement.
- `Commit -> Requirement`: Commits are implementation details; they cannot source business intent.
- `Release -> Architecture`: A release is a snapshot, not a structural definition.
- `Outcome -> Requirement`: Observability outcomes do not map directly to original intent without passing through decisions.

## Memory Promotion Rules

Knowledge enters ARES at varying confidence levels. Promotion to authoritative status is governed by these tiers:

1. **Facts -> Auto Captured**: Repositories, files, and commits are extracted automatically.
2. **Candidates -> Human Governed**: Requirements, Decisions, and Architecture nodes extracted by intelligence engines exist as `Candidates` and require governance/approval.
3. **Memory Nodes -> Authoritative**: Only `Approved` candidates become embedded as definitive `GraphNode` records inside the semantic memory system.

---
**Baseline:** v1.8.1-intelligence-baseline
