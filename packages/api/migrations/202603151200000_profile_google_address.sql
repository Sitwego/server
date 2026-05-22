-- Add Google OAuth fields to profile
ALTER TABLE profile
    ADD COLUMN google_linked BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN google_email VARCHAR(255);

-- Profile address table
CREATE TABLE profile_address (
    id VARCHAR(26) PRIMARY KEY NOT NULL,
    profile_id VARCHAR(26) NOT NULL REFERENCES profile(id) ON DELETE CASCADE,
    street VARCHAR(255),
    city VARCHAR(255),
    state VARCHAR(255),
    zip VARCHAR(20),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_profile_address_profile_id ON profile_address(profile_id);

CREATE OR REPLACE FUNCTION update_profile_address_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_profile_address_updated_at
BEFORE UPDATE ON profile_address
FOR EACH ROW EXECUTE FUNCTION update_profile_address_updated_at();
