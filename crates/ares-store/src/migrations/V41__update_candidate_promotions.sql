-- Add promoted_by and promotion_reason to candidate_promotions
ALTER TABLE candidate_promotions ADD COLUMN promoted_by TEXT NOT NULL DEFAULT 'system';
ALTER TABLE candidate_promotions ADD COLUMN promotion_reason TEXT;
