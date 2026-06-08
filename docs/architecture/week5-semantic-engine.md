# Week 5 — Semantic Memory Engine

Transform ARES from a keyword-based memory system into a semantic memory operating system.

## Architecture

![Semantic Engine Pipeline](https://example.com/placeholder-diagram.png)

### 1. The `ares-core` Traits
To decouple the business logic from specific embedding providers or databases, we define two traits in `ares-core`:
- `EmbeddingProvider`: abstract embedding generator.
- `VectorRepository`: abstract vector database interface.

### 2. The `ares-embeddings` Crate
This new crate houses the HTTP clients and provider integrations, keeping `ares-core` dependency-light.
- **Mock Provider**: Default, deterministic, offline-capable hashing provider.
- **OpenAI Provider**: Optional, uses `reqwest` and `OPENAI_API_KEY`.
- **Ollama Provider**: Optional, uses local `/api/embeddings` endpoint.

### 3. Vector Storage
Stored in a new SQLite table `memory_embeddings` (Migration V8). Includes metadata to prevent mismatched embeddings during re-indexing.
Embeddings are encoded as Little-Endian BLOBs for performance.

### 4. Hybrid Ranking Engine
Combines semantic, keyword, importance, recency, and graph connectivity scores with configurable weights:
- Semantic (0.40)
- Keyword (0.25)
- Importance (0.15)
- Recency (0.10)
- Graph (0.10)

## Future Upgrades
- Replace SQLite Vector Repo with Qdrant, LanceDB, or pgvector using the `VectorRepository` abstraction.
- Run embedding generation via a background task worker.
