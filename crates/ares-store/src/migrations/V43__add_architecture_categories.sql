-- Add architecture candidate fields to candidates table

ALTER TABLE candidates ADD COLUMN architecture_category TEXT;
ALTER TABLE candidates ADD COLUMN dependent_components TEXT DEFAULT '';
ALTER TABLE candidates ADD COLUMN ownership_domains TEXT DEFAULT '';
