-- Candidate lifecycle tables for Requirement/Decision Intelligence

CREATE TABLE IF NOT EXISTS candidates (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    candidate_type TEXT NOT NULL, -- 'Requirement', 'Decision', 'Architecture'
    status TEXT NOT NULL, -- 'Proposed', 'UnderReview', 'Approved', 'Rejected', 'Superseded'
    
    -- Confidence Scores
    evidence_count INTEGER NOT NULL DEFAULT 0,
    source_diversity INTEGER NOT NULL DEFAULT 0,
    temporal_consistency REAL NOT NULL DEFAULT 0.0,
    cluster_strength REAL NOT NULL DEFAULT 0.0,
    
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,

    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS candidate_sources (
    id TEXT PRIMARY KEY,
    candidate_id TEXT NOT NULL,
    source_type TEXT NOT NULL, -- e.g., 'commit', 'file_change'
    source_id TEXT NOT NULL,
    confidence REAL NOT NULL DEFAULT 1.0,

    FOREIGN KEY (candidate_id) REFERENCES candidates(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS candidate_reviews (
    id TEXT PRIMARY KEY,
    candidate_id TEXT NOT NULL,
    reviewer TEXT NOT NULL,
    comment TEXT NOT NULL,
    status_changed_to TEXT NOT NULL,
    review_date BIGINT NOT NULL,

    FOREIGN KEY (candidate_id) REFERENCES candidates(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS candidate_promotions (
    id TEXT PRIMARY KEY,
    candidate_id TEXT NOT NULL,
    promoted_node_id TEXT NOT NULL,
    promoted_at BIGINT NOT NULL,

    FOREIGN KEY (candidate_id) REFERENCES candidates(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_candidates_project ON candidates(project_id);
CREATE INDEX IF NOT EXISTS idx_candidates_status ON candidates(status);
CREATE INDEX IF NOT EXISTS idx_candidate_sources_candidate ON candidate_sources(candidate_id);
CREATE INDEX IF NOT EXISTS idx_candidate_reviews_candidate ON candidate_reviews(candidate_id);
CREATE INDEX IF NOT EXISTS idx_candidate_promotions_candidate ON candidate_promotions(candidate_id);
