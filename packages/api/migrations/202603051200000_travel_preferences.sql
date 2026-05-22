-- Add travel_preferences JSONB column to profile table
ALTER TABLE profile
    ADD COLUMN IF NOT EXISTS travel_preferences JSONB NOT NULL DEFAULT '{}';

-- GIN index for efficient JSONB containment (@>) queries
CREATE INDEX IF NOT EXISTS idx_profile_travel_preferences_gin
    ON profile USING GIN (travel_preferences);

-- To reverse this migration:
-- DROP INDEX IF EXISTS idx_profile_travel_preferences_gin;
-- ALTER TABLE profile DROP COLUMN IF EXISTS travel_preferences;
