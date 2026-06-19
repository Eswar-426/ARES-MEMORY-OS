CREATE TABLE IF NOT EXISTS governance_approval_requests (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    workflow_id TEXT,
    
    status TEXT NOT NULL,
    requested_by TEXT NOT NULL,
    approved_by TEXT,
    
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    expires_at INTEGER,
    
    violations_json TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_gov_appr_project ON governance_approval_requests(project_id);
CREATE INDEX IF NOT EXISTS idx_gov_appr_status ON governance_approval_requests(status);
