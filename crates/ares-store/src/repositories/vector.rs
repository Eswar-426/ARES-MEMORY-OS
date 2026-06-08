//! SQLite-backed vector repository.
//!
//! Implements `ares_core::VectorRepository` using the `memory_embeddings` table.
//! Embeddings are stored as little-endian `f32` BLOBs for compact, efficient storage.

use crate::db::Store;
use ares_core::vector::{
    similarity::{cosine_similarity, SimilarityResult},
    traits::VectorRepository,
    types::{EmbeddingMetadata, StoredEmbedding},
};
use ares_core::AresError;
use rusqlite::params;
use tracing::debug;

/// SQLite implementation of `VectorRepository`.
///
/// Uses brute-force cosine similarity for search.  The architecture supports
/// future replacement with ANN indices (HNSW, FAISS, Qdrant, etc.) by
/// implementing the `VectorRepository` trait on a different struct.
pub struct SqliteVectorRepository {
    store: Store,
}

impl SqliteVectorRepository {
    pub fn new(store: Store) -> Self {
        Self { store }
    }
}

// ─────────────────── Serialization Helpers ───────────────────

/// Encode a `Vec<f32>` as a little-endian byte blob.
fn encode_embedding(embedding: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(embedding.len() * 4);
    for &val in embedding {
        bytes.extend_from_slice(&val.to_le_bytes());
    }
    bytes
}

/// Decode a little-endian byte blob back into `Vec<f32>`.
fn decode_embedding(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|chunk| {
            let arr: [u8; 4] = chunk.try_into().expect("chunk is exactly 4 bytes");
            f32::from_le_bytes(arr)
        })
        .collect()
}

// ─────────────────── VectorRepository Impl ───────────────────

impl VectorRepository for SqliteVectorRepository {
    fn upsert_embedding(
        &self,
        memory_id: &str,
        embedding: &[f32],
        metadata: &EmbeddingMetadata,
    ) -> Result<(), AresError> {
        let blob = encode_embedding(embedding);
        let conn = self.store.get_conn()?;

        conn.execute(
            "INSERT INTO memory_embeddings
               (memory_id, embedding, provider, model, dimensions, embedding_version)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(memory_id) DO UPDATE SET
               embedding         = excluded.embedding,
               provider          = excluded.provider,
               model             = excluded.model,
               dimensions        = excluded.dimensions,
               embedding_version = excluded.embedding_version,
               created_at        = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
            params![
                memory_id,
                blob,
                metadata.provider,
                metadata.model,
                metadata.dimensions,
                metadata.embedding_version,
            ],
        )
        .map_err(AresError::db)?;

        debug!(memory_id = %memory_id, dims = metadata.dimensions, "Embedding upserted");
        Ok(())
    }

    fn get_embedding(&self, memory_id: &str) -> Result<Option<StoredEmbedding>, AresError> {
        let conn = self.store.get_conn()?;
        let result = conn.query_row(
            "SELECT memory_id, embedding, provider, model, dimensions, embedding_version, created_at
             FROM memory_embeddings WHERE memory_id = ?1",
            params![memory_id],
            |row| {
                let blob: Vec<u8> = row.get(1)?;
                Ok(StoredEmbedding {
                    memory_id: row.get(0)?,
                    embedding: decode_embedding(&blob),
                    provider: row.get(2)?,
                    model: row.get(3)?,
                    dimensions: row.get(4)?,
                    embedding_version: row.get(5)?,
                    created_at: row.get(6)?,
                })
            },
        );

        match result {
            Ok(se) => Ok(Some(se)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AresError::db(e)),
        }
    }

    fn delete_embedding(&self, memory_id: &str) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        conn.execute(
            "DELETE FROM memory_embeddings WHERE memory_id = ?1",
            params![memory_id],
        )
        .map_err(AresError::db)?;
        debug!(memory_id = %memory_id, "Embedding deleted");
        Ok(())
    }

    fn search_similar(
        &self,
        query_embedding: &[f32],
        metadata: &EmbeddingMetadata,
        limit: usize,
    ) -> Result<Vec<SimilarityResult>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare("SELECT memory_id, embedding FROM memory_embeddings WHERE provider = ?1 AND model = ?2")
            .map_err(AresError::db)?;

        let rows = stmt
            .query_map(params![metadata.provider, metadata.model], |row| {
                let memory_id: String = row.get(0)?;
                let blob: Vec<u8> = row.get(1)?;
                Ok((memory_id, blob))
            })
            .map_err(AresError::db)?;

        // Compute cosine similarity for every stored embedding (brute-force).
        // Future: replace with ANN index for O(log n) search.
        let mut results: Vec<SimilarityResult> = Vec::new();
        for row in rows {
            let (memory_id, blob) = row.map_err(AresError::db)?;
            let stored = decode_embedding(&blob);
            let score = cosine_similarity(query_embedding, &stored);
            results.push(SimilarityResult { memory_id, score });
        }

        // Sort descending by score and take top N
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);

        Ok(results)
    }

    fn count(&self) -> Result<u64, AresError> {
        let conn = self.store.get_conn()?;
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM memory_embeddings", [], |row| {
                row.get(0)
            })
            .map_err(AresError::db)?;
        Ok(count as u64)
    }

    fn list_embedded_memory_ids(&self) -> Result<Vec<String>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare("SELECT memory_id FROM memory_embeddings ORDER BY created_at DESC")
            .map_err(AresError::db)?;

        let rows = stmt
            .query_map([], |row| row.get(0))
            .map_err(AresError::db)?;

        rows.collect::<Result<Vec<String>, _>>()
            .map_err(AresError::db)
    }
}

// ─────────────────── Tests ───────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_helpers::test_store;

    fn make_metadata() -> EmbeddingMetadata {
        EmbeddingMetadata {
            provider: "mock".to_string(),
            model: "mock-128d".to_string(),
            dimensions: 4,
            embedding_version: 1,
        }
    }

    #[test]
    fn upsert_and_get_embedding() {
        let (store, _dir) = test_store();
        let repo = SqliteVectorRepository::new(store);
        let vec = vec![0.1, 0.2, 0.3, 0.4];
        let meta = make_metadata();

        repo.upsert_embedding("mem_1", &vec, &meta).unwrap();
        let stored = repo.get_embedding("mem_1").unwrap().unwrap();

        assert_eq!(stored.memory_id, "mem_1");
        assert_eq!(stored.embedding.len(), 4);
        assert!((stored.embedding[0] - 0.1).abs() < 1e-6);
        assert_eq!(stored.provider, "mock");
        assert_eq!(stored.model, "mock-128d");
        assert_eq!(stored.dimensions, 4);
        assert_eq!(stored.embedding_version, 1);
    }

    #[test]
    fn upsert_replaces_existing() {
        let (store, _dir) = test_store();
        let repo = SqliteVectorRepository::new(store);
        let meta = make_metadata();

        repo.upsert_embedding("mem_1", &[1.0, 0.0, 0.0, 0.0], &meta)
            .unwrap();
        repo.upsert_embedding("mem_1", &[0.0, 1.0, 0.0, 0.0], &meta)
            .unwrap();

        let stored = repo.get_embedding("mem_1").unwrap().unwrap();
        assert!((stored.embedding[0] - 0.0).abs() < 1e-6);
        assert!((stored.embedding[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn delete_removes_embedding() {
        let (store, _dir) = test_store();
        let repo = SqliteVectorRepository::new(store);
        let meta = make_metadata();

        repo.upsert_embedding("mem_1", &[1.0, 0.0, 0.0, 0.0], &meta)
            .unwrap();
        repo.delete_embedding("mem_1").unwrap();

        assert!(repo.get_embedding("mem_1").unwrap().is_none());
    }

    #[test]
    fn get_nonexistent_returns_none() {
        let (store, _dir) = test_store();
        let repo = SqliteVectorRepository::new(store);
        assert!(repo.get_embedding("nonexistent").unwrap().is_none());
    }

    #[test]
    fn search_similar_returns_ranked_results() {
        let (store, _dir) = test_store();
        let repo = SqliteVectorRepository::new(store);
        let meta = make_metadata();

        // Insert three vectors
        repo.upsert_embedding("mem_close", &[1.0, 0.0, 0.0, 0.0], &meta)
            .unwrap();
        repo.upsert_embedding("mem_medium", &[0.7, 0.7, 0.0, 0.0], &meta)
            .unwrap();
        repo.upsert_embedding("mem_far", &[0.0, 0.0, 0.0, 1.0], &meta)
            .unwrap();

        // Query is closest to mem_close
        let results = repo
            .search_similar(&[1.0, 0.0, 0.0, 0.0], &meta, 10)
            .unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].memory_id, "mem_close");
        assert!((results[0].score - 1.0).abs() < 1e-6);
    }

    #[test]
    fn search_respects_limit() {
        let (store, _dir) = test_store();
        let repo = SqliteVectorRepository::new(store);
        let meta = make_metadata();

        for i in 0..5 {
            repo.upsert_embedding(&format!("mem_{i}"), &[i as f32, 0.0, 0.0, 0.0], &meta)
                .unwrap();
        }

        let results = repo
            .search_similar(&[1.0, 0.0, 0.0, 0.0], &meta, 2)
            .unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn count_returns_correct_value() {
        let (store, _dir) = test_store();
        let repo = SqliteVectorRepository::new(store);
        let meta = make_metadata();

        assert_eq!(repo.count().unwrap(), 0);
        repo.upsert_embedding("mem_1", &[1.0, 0.0, 0.0, 0.0], &meta)
            .unwrap();
        assert_eq!(repo.count().unwrap(), 1);
        repo.upsert_embedding("mem_2", &[0.0, 1.0, 0.0, 0.0], &meta)
            .unwrap();
        assert_eq!(repo.count().unwrap(), 2);
    }

    #[test]
    fn list_embedded_memory_ids() {
        let (store, _dir) = test_store();
        let repo = SqliteVectorRepository::new(store);
        let meta = make_metadata();

        repo.upsert_embedding("mem_a", &[1.0, 0.0, 0.0, 0.0], &meta)
            .unwrap();
        repo.upsert_embedding("mem_b", &[0.0, 1.0, 0.0, 0.0], &meta)
            .unwrap();

        let ids = repo.list_embedded_memory_ids().unwrap();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"mem_a".to_string()));
        assert!(ids.contains(&"mem_b".to_string()));
    }

    #[test]
    fn encode_decode_roundtrip() {
        let original = vec![0.1_f32, -0.5, 1.23, 0.0, f32::MAX, f32::MIN];
        let blob = encode_embedding(&original);
        let decoded = decode_embedding(&blob);
        for (a, b) in original.iter().zip(decoded.iter()) {
            assert_eq!(a.to_bits(), b.to_bits(), "Roundtrip must be bit-exact");
        }
    }

    #[test]
    fn search_empty_database() {
        let (store, _dir) = test_store();
        let repo = SqliteVectorRepository::new(store);
        let meta = make_metadata();
        let results = repo.search_similar(&[1.0, 0.0, 0.0], &meta, 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn search_prevents_incompatible_model_comparisons() {
        let (store, _dir) = test_store();
        let repo = SqliteVectorRepository::new(store);
        let meta1 = make_metadata();
        let meta2 = EmbeddingMetadata {
            provider: "openai".to_string(),
            model: "text-embedding-3-small".to_string(),
            dimensions: 4,
            embedding_version: 1,
        };

        // Insert mock
        repo.upsert_embedding("mem_mock", &[1.0, 0.0, 0.0, 0.0], &meta1)
            .unwrap();
        // Insert openai
        repo.upsert_embedding("mem_openai", &[1.0, 0.0, 0.0, 0.0], &meta2)
            .unwrap();

        // Searching with mock metadata should ONLY return mock
        let results_mock = repo
            .search_similar(&[1.0, 0.0, 0.0, 0.0], &meta1, 10)
            .unwrap();
        assert_eq!(results_mock.len(), 1);
        assert_eq!(results_mock[0].memory_id, "mem_mock");

        // Searching with openai metadata should ONLY return openai
        let results_openai = repo
            .search_similar(&[1.0, 0.0, 0.0, 0.0], &meta2, 10)
            .unwrap();
        assert_eq!(results_openai.len(), 1);
        assert_eq!(results_openai[0].memory_id, "mem_openai");
    }
}
