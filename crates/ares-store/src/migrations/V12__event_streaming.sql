-- V12__event_streaming.sql
-- Core Event Store
CREATE TABLE IF NOT EXISTS event_store (
    id TEXT PRIMARY KEY,
    topic TEXT NOT NULL,
    event_type TEXT NOT NULL,
    source TEXT NOT NULL,
    schema_version INTEGER NOT NULL,
    event_version INTEGER NOT NULL,
    correlation_id TEXT NOT NULL,
    causation_id TEXT,
    trace_id TEXT,
    partition_key TEXT,
    payload TEXT NOT NULL,
    metadata TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE INDEX idx_event_store_topic ON event_store(topic);
CREATE INDEX idx_event_store_event_type ON event_store(event_type);
CREATE INDEX idx_event_store_correlation ON event_store(correlation_id);
CREATE INDEX idx_event_store_timestamp ON event_store(timestamp);
CREATE INDEX idx_event_store_partition ON event_store(partition_key);

-- Event Aggregates
CREATE TABLE IF NOT EXISTS event_aggregates (
    id TEXT PRIMARY KEY,
    aggregate_type TEXT NOT NULL,
    version INTEGER NOT NULL,
    state TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
CREATE INDEX idx_event_aggregates_type ON event_aggregates(aggregate_type);

-- Event Snapshots
CREATE TABLE IF NOT EXISTS event_snapshots (
    id TEXT PRIMARY KEY,
    aggregate_id TEXT NOT NULL,
    aggregate_type TEXT NOT NULL,
    version INTEGER NOT NULL,
    snapshot_data TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY(aggregate_id) REFERENCES event_aggregates(id) ON DELETE CASCADE
);
CREATE INDEX idx_event_snapshots_aggregate ON event_snapshots(aggregate_id);

-- Consumer Groups
CREATE TABLE IF NOT EXISTS event_consumer_groups (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    topic_pattern TEXT NOT NULL,
    status TEXT NOT NULL, -- active, paused
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Group Offsets
CREATE TABLE IF NOT EXISTS event_group_offsets (
    group_id TEXT NOT NULL,
    partition_key TEXT NOT NULL,
    last_processed_event_id TEXT,
    last_processed_timestamp INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (group_id, partition_key),
    FOREIGN KEY(group_id) REFERENCES event_consumer_groups(id) ON DELETE CASCADE
);

-- Event DLQ
CREATE TABLE IF NOT EXISTS event_dlq (
    id TEXT PRIMARY KEY,
    event_id TEXT NOT NULL,
    group_id TEXT,
    error_message TEXT NOT NULL,
    stack_trace TEXT,
    retry_count INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL, -- pending, resolved, discarded
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY(event_id) REFERENCES event_store(id) ON DELETE CASCADE
);
CREATE INDEX idx_event_dlq_status ON event_dlq(status);

-- Event Streams (Logical streams that might group multiple topics)
CREATE TABLE IF NOT EXISTS event_streams (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Event Subscriptions (Users/Systems subscribed to topics/streams)
CREATE TABLE IF NOT EXISTS event_subscriptions (
    id TEXT PRIMARY KEY,
    subscriber_id TEXT NOT NULL,
    topic_filter TEXT NOT NULL,
    status TEXT NOT NULL, -- active, paused, deleted
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Delivery Log (Tracking deliveries to active subscriptions/transports)
CREATE TABLE IF NOT EXISTS event_delivery_log (
    id TEXT PRIMARY KEY,
    event_id TEXT NOT NULL,
    subscription_id TEXT NOT NULL,
    delivery_status TEXT NOT NULL, -- delivered, failed
    attempt_count INTEGER NOT NULL,
    last_error TEXT,
    delivered_at INTEGER,
    created_at INTEGER NOT NULL,
    FOREIGN KEY(event_id) REFERENCES event_store(id) ON DELETE CASCADE
);

-- Replay Log (Audit log for replay jobs)
CREATE TABLE IF NOT EXISTS event_replay_log (
    id TEXT PRIMARY KEY,
    replay_job_id TEXT NOT NULL,
    target_topic TEXT,
    start_time INTEGER,
    end_time INTEGER,
    events_replayed INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL, -- running, completed, failed
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
