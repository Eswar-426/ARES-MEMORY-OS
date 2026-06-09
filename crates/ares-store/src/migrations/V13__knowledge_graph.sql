-- Phase A Core Tables
CREATE TABLE graph_entities (
    id TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    properties TEXT NOT NULL DEFAULT '{}',
    embedding BLOB,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    valid_from TEXT,
    valid_to TEXT,
    confidence_score REAL DEFAULT 1.0,
    source_event_id TEXT
);

CREATE TABLE graph_relationships (
    id TEXT PRIMARY KEY,
    source_entity TEXT NOT NULL,
    target_entity TEXT NOT NULL,
    relationship_type TEXT NOT NULL,
    properties TEXT NOT NULL DEFAULT '{}',
    embedding BLOB,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    valid_from TEXT,
    valid_to TEXT,
    confidence_score REAL DEFAULT 1.0,
    evidence_count INTEGER DEFAULT 1,
    source_event_id TEXT,
    FOREIGN KEY (source_entity) REFERENCES graph_entities(id) ON DELETE CASCADE,
    FOREIGN KEY (target_entity) REFERENCES graph_entities(id) ON DELETE CASCADE
);

CREATE TABLE graph_embeddings (
    id TEXT PRIMARY KEY,
    target_id TEXT NOT NULL, -- references either entity or relationship
    target_type TEXT NOT NULL, -- 'ENTITY' or 'RELATIONSHIP'
    embedding BLOB NOT NULL,
    model TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE graph_versions (
    id TEXT PRIMARY KEY,
    version_name TEXT NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT
);

CREATE TABLE entity_aliases (
    id TEXT PRIMARY KEY,
    entity_id TEXT NOT NULL,
    alias TEXT NOT NULL,
    normalized_alias TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (entity_id) REFERENCES graph_entities(id) ON DELETE CASCADE
);

CREATE TABLE knowledge_events (
    id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,
    payload TEXT NOT NULL,
    processed_at TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'PENDING'
);

CREATE TABLE knowledge_projections (
    id TEXT PRIMARY KEY,
    projection_type TEXT NOT NULL,
    projection_key TEXT NOT NULL,
    data TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    updated_at TEXT NOT NULL,
    UNIQUE(projection_type, projection_key)
);

CREATE TABLE knowledge_cache (
    id TEXT PRIMARY KEY,
    cache_key TEXT NOT NULL UNIQUE,
    data TEXT NOT NULL,
    expires_at TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE goal_states (
    id TEXT PRIMARY KEY,
    entity_id TEXT NOT NULL,
    status TEXT NOT NULL,
    progress REAL NOT NULL DEFAULT 0.0,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (entity_id) REFERENCES graph_entities(id) ON DELETE CASCADE
);

-- Phase B & C Extra Tables for complete schema V13
CREATE TABLE graph_traversals (
    id TEXT PRIMARY KEY,
    start_entity TEXT NOT NULL,
    path TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE graph_communities (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    metadata TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE graph_snapshots (
    id TEXT PRIMARY KEY,
    snapshot_data BLOB NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE knowledge_provenance (
    id TEXT PRIMARY KEY,
    entity_id TEXT,
    relationship_id TEXT,
    event_id TEXT NOT NULL,
    source_type TEXT NOT NULL,
    created_by TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE graph_constraints (
    id TEXT PRIMARY KEY,
    constraint_type TEXT NOT NULL,
    rules TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE graph_exports (
    id TEXT PRIMARY KEY,
    format TEXT NOT NULL,
    data BLOB NOT NULL,
    created_at TEXT NOT NULL
);

-- Indexes
CREATE INDEX idx_graph_entities_type ON graph_entities(entity_type);
CREATE INDEX idx_graph_entities_name ON graph_entities(name);
CREATE INDEX idx_graph_relationships_type ON graph_relationships(relationship_type);
CREATE INDEX idx_graph_relationships_source ON graph_relationships(source_entity);
CREATE INDEX idx_graph_relationships_target ON graph_relationships(target_entity);
CREATE INDEX idx_knowledge_events_type ON knowledge_events(event_type);
CREATE INDEX idx_graph_versions_created ON graph_versions(created_at);
CREATE INDEX idx_entity_aliases_normalized ON entity_aliases(normalized_alias);
