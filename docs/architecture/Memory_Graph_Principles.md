# ARES Memory Graph Principles

## Vision
ARES is a **Repository Memory Operating System**, not a generic graph database. The Repository Knowledge Graph is designed explicitly as a **Memory Graph**. 

Every node and edge within this graph exists to answer the core questions of ARES:
- *Why does this exist?*
- *Who owns it?*
- *What approved it?*
- *What evidence supports it?*
- *What breaks if it changes?*
- *How has it evolved?*
- *What knowledge debt surrounds it?*

## The Canonical Hierarchy
The memory graph strictly adheres to the following sequence. Every current and future node type must fit somewhere within this hierarchy:

```
Requirements
      ↓
Decisions
      ↓
Architecture
      ↓
Code Artifacts
      ↓
Tests
      ↓
Runtime Signals
      ↓
Outcomes
```

Parallel to this primary execution hierarchy is the **Intelligence and Maintenance Hierarchy**:

```
Gaps
      ↓
Root Causes
      ↓
Resolutions
```

## Guiding Principles

1. **Event-Driven Materialization**  
   Domains never insert directly into the graph. The graph is projected through continuous synchronization of domain events (e.g., `RequirementCreated`, `DecisionApproved`). The domains remain bounded contexts, and the graph acts as an omniscient observer.

2. **Immutable Evolution (Versioning)**  
   Memory evolves, and the history of that evolution is valuable. The graph tracks versions, mapping state changes over time so that historical context is never lost.

3. **Traceability as a Superpower**  
   Paths must easily traverse from `Outcomes` back up to `Requirements`, and from `Code Artifacts` down to `Resolutions`. Impact radius computing (e.g., "What breaks if this changes?") relies on these clean, unidirectional dependencies represented by edge types.

4. **Expressive Relationships**  
   Edges describe the exact nature of a connection (e.g., `Implements`, `Drives`, `Validates`, `Exhibits`). Ambiguous or generic links are not permitted.
