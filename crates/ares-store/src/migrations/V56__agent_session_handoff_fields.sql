-- Add handoff fields for agent session chaining (ares_end_session → ares_briefing)
ALTER TABLE agent_sessions ADD COLUMN left_incomplete TEXT NOT NULL DEFAULT '';
ALTER TABLE agent_sessions ADD COLUMN recommended_next TEXT NOT NULL DEFAULT '';
