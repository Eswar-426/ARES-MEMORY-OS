# ARES Context Engine Architecture (v1.0)

## Purpose
The Context Engine is the central retrieval, traversal, and ranking layer of ARES. It transforms raw repository knowledge—extracted by the `ares-scanner` and persisted in the `ares-store`—into structured, high-value `ContextPacks` and `ContextBundles`. It serves as the primary bridge between the static knowledge graph and intelligent agents, MCP clients, or other consumers requiring deterministic context resolution.

## Responsibilities
- **Intent Detection**: Map natural language queries or structured requests into predefined query intents deterministically without using an LLM.
- **Graph Traversal**: Navigate the repository knowledge graph to find definitions, dependencies, entry points, neighbors, and shortest paths.
- **Context Retrieval**: Combine graph knowledge, repository summaries, historical snapshots, and project memories into a unified dataset.
- **Impact Analysis**: Identify the blast radius of changes to specific modules or functions.
- **Ranking**: Score and filter retrieved nodes based on distance, recency, relationship strength, and architectural importance.
- **Packaging**: Assemble the final data into a `ContextPack` consumable by LLM Agents.

## Architecture Diagram

```text
                ┌─────────────────┐
                │ User Question   │
                └────────┬────────┘
                         │
                         ▼
                ┌─────────────────┐
                │ Intent Detector │
                └────────┬────────┘
                         │
                         ▼
                ┌─────────────────┐
                │ ContextRetriever│
                └────────┬────────┘
                         │
       ┌─────────────────┼─────────────────┐
       ▼                 ▼                 ▼
┌────────────┐   ┌────────────┐   ┌────────────┐
│Knowledge   │   │Project     │   │Repository  │
│Graph       │   │Memory      │   │Summary     │
└────────────┘   └────────────┘   └────────────┘
       │                 │                 │
       └─────────────────┼─────────────────┘
                         ▼
                ┌─────────────────┐
                │ GraphTraversal  │
                └────────┬────────┘
                         ▼
                ┌─────────────────┐
                │ Ranking Engine  │
                └────────┬────────┘
                         ▼
                ┌─────────────────┐
                │ Context Bundle  │
                └────────┬────────┘
                         ▼
                ┌─────────────────┐
                │ Context Pack    │
                └────────┬────────┘
                         ▼
                ┌─────────────────┐
                │ Agents / MCP    │
                └─────────────────┘
```

## Future MCP Integration Hooks
```text
MCP Client
      │
      ▼
ARES Context Engine
      │
      ▼
Context Pack
      │
      ▼
LLM Agent
```
The design guarantees forward compatibility by encapsulating `ContextBundle` within a `ContextPack` containing actionable summaries, ranked nodes, and observable metadata.

## Data Flow
1. **Query Input**: The user or agent submits a question or search query.
2. **Intent Parsing**: `IntentDetector` applies deterministic heuristics (keywords, regex patterns) to map the query to a structured `QueryIntent`.
3. **Retrieval**: Based on intent, the `ContextRetriever` calls `ares-store` to load graph seeds, project memories, or repository summaries.
4. **Traversal**: `GraphTraversalEngine` explores the graph from the seed nodes up to a configurable `max_depth` or `max_neighbors`.
5. **Ranking**: Nodes discovered are weighted by `RankingStrategy` (e.g., hybrid scoring of distance and recency).
6. **Packaging**: The highest-scoring elements are compiled into a `ContextBundle`.
7. **Metrics**: Performance metadata (retrieval time, nodes examined) is calculated.
8. **Delivery**: The final `ContextPack` is returned.

## Retrieval Pipeline
The retriever fetches isolated pieces of data:
- `ares_store::SqliteGraphRepository` -> AST nodes and structural relationships.
- `ares_store::SqliteMemoryRepository` -> Architectural decisions and features.
- `ares_project_memory` -> High-level project snapshots.

## Ranking Pipeline
Ranking is decoupled via the `RankingStrategy` trait:
- `DistanceScorer`: Penalizes nodes based on graph edge distance from the seed.
- `RecencyScorer`: Boosts nodes recently modified or created.
- `HybridRanker`: Combines multiple strategies using weighted formulas.

## Query Flow
**Supported Intents:**
- `FILE_EXPLANATION`
- `DEPENDENCY_TRACE`
- `ARCHITECTURE_QUERY`
- `COMPONENT_OWNER`
- `CHANGE_IMPACT`
- `DEAD_CODE_DISCOVERY`
- `ENTRY_POINT_DISCOVERY`
- `MEMORY_LOOKUP`
- `REPOSITORY_OVERVIEW`
- `IMPLEMENTATION_SEARCH`

## Performance Considerations
Graph operations can exponentially expand. A `TraversalConfig` is enforced across all navigations:
```rust
pub struct TraversalConfig {
    pub max_depth: usize,       // default: 5
    pub max_neighbors: usize,   // default: 100
    pub max_results: usize,     // default: 50
}
```
All queries utilize async `tokio` channels or futures where parallel IO against SQLite is supported, maintaining low overhead.

## Extension Points
- `IntentDetector` can be upgraded from heuristics to local lightweight ML classification models in the future.
- `RankingStrategy` can implement complex vector-similarity scoring when embeddings are introduced.
- `ContextPack` includes a flexible `ContextMetadata` dictionary for arbitrary telemetry.

## Example Requests
- "Trace dependencies for `auth.rs`" -> `DEPENDENCY_TRACE`
- "What modules will be affected if I change `parser.rs`?" -> `CHANGE_IMPACT`

## Example Responses
A `ContextPack` containing a `ContextBundle` populated with an `ImpactReport` identifying 4 downstream modules and 12 affected functions.

## Failure Modes
- **Orphan Nodes**: Traversal stops if nodes lack relationship edges (e.g., dynamically resolved imports not statically identifiable).
- **Ambiguous Queries**: Defaults to `IMPLEMENTATION_SEARCH` or `REPOSITORY_OVERVIEW` if intent heuristics fail to trigger.

## Testing Strategy
- **Unit Tests**: Test intent heuristics against a large suite of mock queries.
- **Integration Tests**: Mock an in-memory SQLite graph with known depths and paths. Assert that the traversal engine halts at `max_depth` and respects `max_neighbors`.
- **Self-Test Binary**: A `context_self_test.rs` binary will run live queries against `ARES_Memory_os` itself.
