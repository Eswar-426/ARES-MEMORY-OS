-- V21: Multi-Agent Coordination & Autonomous Organization Layer
-- Week 19 — Tables for the coordinated multi-agent operating system

-- 1. Agent teams
CREATE TABLE IF NOT EXISTS agent_teams (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    leader_id   TEXT,
    goal        TEXT NOT NULL DEFAULT '',
    strategy    TEXT NOT NULL DEFAULT 'parallel',
    status      TEXT NOT NULL DEFAULT 'forming',
    created_at  INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_agent_teams_status ON agent_teams(status);

-- 2. Agent team members
CREATE TABLE IF NOT EXISTS agent_team_members (
    team_id     TEXT NOT NULL,
    agent_id    TEXT NOT NULL,
    role        TEXT NOT NULL,
    joined_at   INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    PRIMARY KEY (team_id, agent_id),
    FOREIGN KEY (team_id) REFERENCES agent_teams(id)
);

-- 3. Agent messages
CREATE TABLE IF NOT EXISTS agent_messages (
    id              TEXT PRIMARY KEY,
    from_id         TEXT NOT NULL,
    to_id           TEXT,
    conversation_id TEXT,
    mission_id      TEXT,
    msg_type        TEXT NOT NULL,
    priority        TEXT NOT NULL DEFAULT 'normal',
    subject         TEXT NOT NULL DEFAULT '',
    payload         TEXT NOT NULL DEFAULT '',
    created_at      INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_agent_messages_from ON agent_messages(from_id);
CREATE INDEX IF NOT EXISTS idx_agent_messages_to ON agent_messages(to_id);
CREATE INDEX IF NOT EXISTS idx_agent_messages_conversation ON agent_messages(conversation_id);

-- 4. Agent conversations
CREATE TABLE IF NOT EXISTS agent_conversations (
    id          TEXT PRIMARY KEY,
    topic       TEXT NOT NULL,
    team_id     TEXT,
    state       TEXT NOT NULL DEFAULT 'active',
    created_at  INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at  INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_agent_conversations_state ON agent_conversations(state);

-- 5. Agent reputation
CREATE TABLE IF NOT EXISTS agent_reputation (
    agent_id        TEXT PRIMARY KEY,
    success_rate    REAL NOT NULL DEFAULT 0.5,
    avg_latency_ms  REAL NOT NULL DEFAULT 0.0,
    reliability     REAL NOT NULL DEFAULT 1.0,
    cost_efficiency REAL NOT NULL DEFAULT 0.5,
    quality_score   REAL NOT NULL DEFAULT 0.5,
    task_count      INTEGER NOT NULL DEFAULT 0,
    updated_at      INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- 6. Agent delegations
CREATE TABLE IF NOT EXISTS agent_delegations (
    id          TEXT PRIMARY KEY,
    task_id     TEXT NOT NULL,
    from_agent  TEXT NOT NULL,
    to_agent    TEXT NOT NULL,
    depth       INTEGER NOT NULL DEFAULT 0,
    status      TEXT NOT NULL DEFAULT 'pending',
    reason      TEXT NOT NULL DEFAULT '',
    result      TEXT,
    created_at  INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    completed_at INTEGER
);

CREATE INDEX IF NOT EXISTS idx_agent_delegations_task ON agent_delegations(task_id);
CREATE INDEX IF NOT EXISTS idx_agent_delegations_status ON agent_delegations(status);

-- 7. Agent consensus rounds
CREATE TABLE IF NOT EXISTS agent_consensus (
    id              TEXT PRIMARY KEY,
    topic           TEXT NOT NULL,
    algorithm       TEXT NOT NULL,
    options_json    TEXT NOT NULL DEFAULT '[]',
    participants_json TEXT NOT NULL DEFAULT '[]',
    votes_json      TEXT NOT NULL DEFAULT '[]',
    result          TEXT,
    state           TEXT NOT NULL DEFAULT 'open',
    created_at      INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    resolved_at     INTEGER
);

-- 8. Agent debates
CREATE TABLE IF NOT EXISTS agent_debates (
    id          TEXT PRIMARY KEY,
    topic       TEXT NOT NULL,
    proposer_id TEXT NOT NULL,
    opponent_id TEXT NOT NULL,
    judge_id    TEXT NOT NULL,
    arguments_json TEXT NOT NULL DEFAULT '[]',
    outcome     TEXT,
    state       TEXT NOT NULL DEFAULT 'awaiting_proposal',
    created_at  INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    resolved_at INTEGER
);

-- 9. Agent conflicts
CREATE TABLE IF NOT EXISTS agent_conflicts (
    id              TEXT PRIMARY KEY,
    conflict_type   TEXT NOT NULL,
    involved_agents_json TEXT NOT NULL DEFAULT '[]',
    description     TEXT NOT NULL DEFAULT '',
    state           TEXT NOT NULL DEFAULT 'detected',
    resolution      TEXT,
    detected_at     INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    resolved_at     INTEGER
);

CREATE INDEX IF NOT EXISTS idx_agent_conflicts_state ON agent_conflicts(state);

-- 10. Swarm executions
CREATE TABLE IF NOT EXISTS swarm_executions (
    id              TEXT PRIMARY KEY,
    strategy        TEXT NOT NULL,
    task_description TEXT NOT NULL DEFAULT '',
    agent_ids_json  TEXT NOT NULL DEFAULT '[]',
    results_json    TEXT NOT NULL DEFAULT '[]',
    best_agent_id   TEXT,
    state           TEXT NOT NULL DEFAULT 'initializing',
    created_at      INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    completed_at    INTEGER
);

-- 11. Organizational learning
CREATE TABLE IF NOT EXISTS org_learning (
    id          TEXT PRIMARY KEY,
    category    TEXT NOT NULL,
    key         TEXT NOT NULL,
    ema_score   REAL NOT NULL DEFAULT 0.5,
    ema_quality REAL NOT NULL DEFAULT 0.5,
    ema_cost    REAL NOT NULL DEFAULT 0.0,
    sample_count INTEGER NOT NULL DEFAULT 0,
    updated_at  INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_org_learning_category ON org_learning(category);
CREATE UNIQUE INDEX IF NOT EXISTS idx_org_learning_category_key ON org_learning(category, key);

-- 12. Resource reservations
CREATE TABLE IF NOT EXISTS resource_reservations (
    id          TEXT PRIMARY KEY,
    task_id     TEXT NOT NULL,
    cpu_slots   INTEGER NOT NULL DEFAULT 0,
    memory_mb   INTEGER NOT NULL DEFAULT 0,
    gpu_slots   INTEGER NOT NULL DEFAULT 0,
    token_budget INTEGER NOT NULL DEFAULT 0,
    tool_slots  INTEGER NOT NULL DEFAULT 0,
    network_slots INTEGER NOT NULL DEFAULT 0,
    created_at  INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    released_at INTEGER
);

-- 13. Governor decisions
CREATE TABLE IF NOT EXISTS governor_decisions (
    id          TEXT PRIMARY KEY,
    check_type  TEXT NOT NULL,
    decision    TEXT NOT NULL,
    reason      TEXT NOT NULL DEFAULT '',
    created_at  INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_governor_decisions_type ON governor_decisions(check_type);
