CREATE TABLE IF NOT EXISTS policy_versions (
    id TEXT PRIMARY KEY,
    policy_name TEXT NOT NULL,
    version TEXT NOT NULL,
    checksum TEXT NOT NULL,
    loaded_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS compliance_results (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    policy_version_id TEXT NOT NULL REFERENCES policy_versions(id),
    compliant BOOLEAN NOT NULL,
    score REAL NOT NULL,
    evaluated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS compliance_violations (
    id TEXT PRIMARY KEY,
    result_id TEXT NOT NULL REFERENCES compliance_results(id) ON DELETE CASCADE,
    severity TEXT NOT NULL,
    policy_name TEXT NOT NULL,
    node_id TEXT NOT NULL,
    reason TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS compliance_violation_supports (
    violation_id TEXT NOT NULL REFERENCES compliance_violations(id) ON DELETE CASCADE,
    node_id TEXT NOT NULL,
    PRIMARY KEY (violation_id, node_id)
);

CREATE TABLE IF NOT EXISTS governance_certifications (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    certified BOOLEAN NOT NULL,
    policy_score REAL NOT NULL,
    violations_count INTEGER NOT NULL,
    ownership_score REAL NOT NULL,
    traceability_score REAL NOT NULL,
    evidence_score REAL NOT NULL,
    approval_score REAL NOT NULL,
    overall_score REAL NOT NULL,
    evaluated_at INTEGER NOT NULL
);
