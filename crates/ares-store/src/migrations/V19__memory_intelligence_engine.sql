-- V19__memory_intelligence_engine.sql
-- Week 17: Memory Intelligence & Knowledge Evolution Engine
-- Creates episodic memory, semantic memory, consolidation, evolution,
-- decision intelligence, experience learning, compression, and retrieval tables.

-- 1. Episodes — mission experience records
CREATE TABLE IF NOT EXISTS episodes (
    id              TEXT PRIMARY KEY,
    mission_id      TEXT NOT NULL,
    title           TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    agents_involved TEXT NOT NULL DEFAULT '[]',   -- JSON array of agent IDs
    decisions_made  TEXT NOT NULL DEFAULT '[]',   -- JSON array of decision summaries
    outcome         TEXT NOT NULL DEFAULT 'unknown'
                    CHECK(outcome IN ('success','partial_success','failure','aborted','unknown')),
    score           REAL NOT NULL DEFAULT 0.0,
    cost            REAL NOT NULL DEFAULT 0.0,
    duration_secs   REAL NOT NULL DEFAULT 0.0,
    failures        TEXT NOT NULL DEFAULT '[]',   -- JSON array of failure descriptions
    lessons_learned TEXT NOT NULL DEFAULT '[]',   -- JSON array of lesson strings
    tags            TEXT NOT NULL DEFAULT '[]',   -- JSON array of tags for similarity search
    created_at      INTEGER NOT NULL,
    completed_at    INTEGER
);

CREATE INDEX IF NOT EXISTS idx_episodes_mission ON episodes(mission_id);
CREATE INDEX IF NOT EXISTS idx_episodes_outcome ON episodes(outcome);
CREATE INDEX IF NOT EXISTS idx_episodes_created ON episodes(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_episodes_score   ON episodes(score DESC);

-- 2. Episode Events — individual events within an episode timeline
CREATE TABLE IF NOT EXISTS episode_events (
    id          TEXT PRIMARY KEY,
    episode_id  TEXT NOT NULL REFERENCES episodes(id) ON DELETE CASCADE,
    event_type  TEXT NOT NULL
                CHECK(event_type IN ('action','decision','error','milestone','observation','reflection')),
    description TEXT NOT NULL,
    agent_id    TEXT,
    timestamp   INTEGER NOT NULL,
    metadata    TEXT NOT NULL DEFAULT '{}'  -- JSON object
);

CREATE INDEX IF NOT EXISTS idx_episode_events_episode ON episode_events(episode_id, timestamp);
CREATE INDEX IF NOT EXISTS idx_episode_events_type    ON episode_events(event_type);

-- 3. Episode Summaries — compressed episode summaries for fast retrieval
CREATE TABLE IF NOT EXISTS episode_summaries (
    id                TEXT PRIMARY KEY,
    episode_id        TEXT NOT NULL REFERENCES episodes(id) ON DELETE CASCADE,
    summary_text      TEXT NOT NULL,
    key_insights      TEXT NOT NULL DEFAULT '[]',   -- JSON array
    compression_ratio REAL NOT NULL DEFAULT 1.0,
    created_at        INTEGER NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_episode_summaries_episode ON episode_summaries(episode_id);

-- 4. Semantic Memories — extracted concepts, entities, relationships
CREATE TABLE IF NOT EXISTS semantic_memories (
    id                TEXT PRIMARY KEY,
    source_episode_id TEXT REFERENCES episodes(id) ON DELETE SET NULL,
    memory_type       TEXT NOT NULL
                      CHECK(memory_type IN ('entity','relationship','fact','concept')),
    subject           TEXT NOT NULL,
    predicate         TEXT NOT NULL DEFAULT '',
    object            TEXT NOT NULL DEFAULT '',
    confidence        REAL NOT NULL DEFAULT 0.5
                      CHECK(confidence >= 0.0 AND confidence <= 1.0),
    evidence_count    INTEGER NOT NULL DEFAULT 1,
    tags              TEXT NOT NULL DEFAULT '[]',
    created_at        INTEGER NOT NULL,
    updated_at        INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_semantic_type       ON semantic_memories(memory_type);
CREATE INDEX IF NOT EXISTS idx_semantic_subject    ON semantic_memories(subject);
CREATE INDEX IF NOT EXISTS idx_semantic_confidence ON semantic_memories(confidence DESC);
CREATE INDEX IF NOT EXISTS idx_semantic_source     ON semantic_memories(source_episode_id);

-- 5. Knowledge Evolution — tracks confidence changes, contradictions, decay
CREATE TABLE IF NOT EXISTS knowledge_evolution (
    id                  TEXT PRIMARY KEY,
    semantic_memory_id  TEXT NOT NULL REFERENCES semantic_memories(id) ON DELETE CASCADE,
    event_type          TEXT NOT NULL
                        CHECK(event_type IN ('confidence_increase','confidence_decrease',
                              'contradiction_detected','decay_applied','reinforcement',
                              'entity_merged','deprecated')),
    old_confidence      REAL NOT NULL,
    new_confidence      REAL NOT NULL,
    reason              TEXT NOT NULL DEFAULT '',
    source_episode_id   TEXT REFERENCES episodes(id) ON DELETE SET NULL,
    created_at          INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_evolution_semantic ON knowledge_evolution(semantic_memory_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_evolution_type     ON knowledge_evolution(event_type);

-- 6. Memory Clusters — groups of related memories from consolidation
CREATE TABLE IF NOT EXISTS memory_clusters (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    cluster_type    TEXT NOT NULL DEFAULT 'topic'
                    CHECK(cluster_type IN ('topic','temporal','causal','similarity')),
    member_count    INTEGER NOT NULL DEFAULT 0,
    centroid_tags   TEXT NOT NULL DEFAULT '[]',   -- JSON array of representative tags
    summary         TEXT NOT NULL DEFAULT '',
    created_at      INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_clusters_type    ON memory_clusters(cluster_type);
CREATE INDEX IF NOT EXISTS idx_clusters_updated ON memory_clusters(updated_at DESC);

-- 6b. Cluster Membership — links episodes to clusters (many-to-many)
CREATE TABLE IF NOT EXISTS cluster_memberships (
    cluster_id  TEXT NOT NULL REFERENCES memory_clusters(id) ON DELETE CASCADE,
    episode_id  TEXT NOT NULL REFERENCES episodes(id) ON DELETE CASCADE,
    similarity  REAL NOT NULL DEFAULT 0.0,
    added_at    INTEGER NOT NULL,
    PRIMARY KEY (cluster_id, episode_id)
);

-- 7. Memory Principles — high-level principles derived from repeated lessons
CREATE TABLE IF NOT EXISTS memory_principles (
    id              TEXT PRIMARY KEY,
    title           TEXT NOT NULL,
    description     TEXT NOT NULL,
    source_lessons  TEXT NOT NULL DEFAULT '[]',   -- JSON array of lesson IDs
    evidence_count  INTEGER NOT NULL DEFAULT 1,
    confidence      REAL NOT NULL DEFAULT 0.5
                    CHECK(confidence >= 0.0 AND confidence <= 1.0),
    domain          TEXT NOT NULL DEFAULT 'general',
    is_active       INTEGER NOT NULL DEFAULT 1,
    created_at      INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_principles_domain     ON memory_principles(domain);
CREATE INDEX IF NOT EXISTS idx_principles_confidence ON memory_principles(confidence DESC);
CREATE INDEX IF NOT EXISTS idx_principles_active     ON memory_principles(is_active);

-- 8. Decision History — detailed decision records with alternatives and outcomes
CREATE TABLE IF NOT EXISTS decision_history (
    id              TEXT PRIMARY KEY,
    episode_id      TEXT REFERENCES episodes(id) ON DELETE SET NULL,
    decision_type   TEXT NOT NULL DEFAULT 'strategic'
                    CHECK(decision_type IN ('strategic','tactical','technical','resource','retry')),
    question        TEXT NOT NULL,
    chosen_option   TEXT NOT NULL,
    alternatives    TEXT NOT NULL DEFAULT '[]',   -- JSON array of {option, reason_rejected}
    reasoning       TEXT NOT NULL DEFAULT '',
    confidence      REAL NOT NULL DEFAULT 0.5,
    outcome         TEXT   -- filled in after execution
                    CHECK(outcome IS NULL OR outcome IN ('positive','negative','neutral','unknown')),
    context         TEXT NOT NULL DEFAULT '{}',   -- JSON metadata
    created_at      INTEGER NOT NULL,
    resolved_at     INTEGER
);

CREATE INDEX IF NOT EXISTS idx_decision_history_episode  ON decision_history(episode_id);
CREATE INDEX IF NOT EXISTS idx_decision_history_type     ON decision_history(decision_type);
CREATE INDEX IF NOT EXISTS idx_decision_history_outcome  ON decision_history(outcome);
CREATE INDEX IF NOT EXISTS idx_decision_history_created  ON decision_history(created_at DESC);

-- 9. Experience Reports — experience→lesson→principle conversion records
CREATE TABLE IF NOT EXISTS experience_reports (
    id              TEXT PRIMARY KEY,
    episode_id      TEXT REFERENCES episodes(id) ON DELETE SET NULL,
    experience_type TEXT NOT NULL DEFAULT 'observation'
                    CHECK(experience_type IN ('observation','success_pattern','failure_pattern',
                          'optimization','workaround')),
    title           TEXT NOT NULL,
    description     TEXT NOT NULL,
    lesson          TEXT NOT NULL DEFAULT '',
    principle_id    TEXT REFERENCES memory_principles(id) ON DELETE SET NULL,
    frequency       INTEGER NOT NULL DEFAULT 1,
    confidence      REAL NOT NULL DEFAULT 0.5,
    created_at      INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_experience_type      ON experience_reports(experience_type);
CREATE INDEX IF NOT EXISTS idx_experience_episode   ON experience_reports(episode_id);
CREATE INDEX IF NOT EXISTS idx_experience_principle ON experience_reports(principle_id);
CREATE INDEX IF NOT EXISTS idx_experience_freq      ON experience_reports(frequency DESC);

-- 10. Memory Retrieval Log — tracks what was retrieved and when
CREATE TABLE IF NOT EXISTS memory_retrieval_log (
    id              TEXT PRIMARY KEY,
    query_text      TEXT NOT NULL,
    query_type      TEXT NOT NULL DEFAULT 'general'
                    CHECK(query_type IN ('similar_mission','failure_search','success_search',
                          'lesson_search','principle_search','general')),
    results_count   INTEGER NOT NULL DEFAULT 0,
    result_ids      TEXT NOT NULL DEFAULT '[]',   -- JSON array of retrieved IDs
    relevance_score REAL NOT NULL DEFAULT 0.0,
    retrieval_ms    INTEGER NOT NULL DEFAULT 0,
    created_at      INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_retrieval_log_type    ON memory_retrieval_log(query_type);
CREATE INDEX IF NOT EXISTS idx_retrieval_log_created ON memory_retrieval_log(created_at DESC);
