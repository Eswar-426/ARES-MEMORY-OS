CREATE TABLE IF NOT EXISTS memory_embeddings (
    memory_id         TEXT PRIMARY KEY,
    embedding         BLOB NOT NULL,
    provider          TEXT NOT NULL DEFAULT 'mock',
    model             TEXT NOT NULL DEFAULT 'mock-128d',
    dimensions        INTEGER NOT NULL,
    embedding_version INTEGER NOT NULL DEFAULT 1,
    created_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- Index for listing/counting embeddings by provider/model
CREATE INDEX IF NOT EXISTS idx_embeddings_provider_model
    ON memory_embeddings (provider, model);
