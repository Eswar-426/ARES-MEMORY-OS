-- Migration to support Traceability Candidates

ALTER TABLE candidates ADD COLUMN traceability_category TEXT;
ALTER TABLE candidates ADD COLUMN source_endpoint_type TEXT;
ALTER TABLE candidates ADD COLUMN source_endpoint_id TEXT;
ALTER TABLE candidates ADD COLUMN target_endpoint_type TEXT;
ALTER TABLE candidates ADD COLUMN target_endpoint_id TEXT;
ALTER TABLE candidates ADD COLUMN traceability_strength TEXT;
