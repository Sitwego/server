CREATE TABLE location (
    id VARCHAR(26) PRIMARY KEY NOT NULL,
    lat DOUBLE PRECISION NOT NULL,
    lon DOUBLE PRECISION NOT NULL,
    street  CHARACTER VARYING(255),
    city  CHARACTER VARYING(255),
    road  CHARACTER VARYING(255),
    state  CHARACTER VARYING(255),
    country  CHARACTER VARYING(255),
    building  CHARACTER VARYING(255),
    floor  CHARACTER VARYING(255),
    door  CHARACTER VARYING(255),
    area_code  CHARACTER VARYING(255),
    area  CHARACTER VARYING(255),
    ward  CHARACTER VARYING(255),
    place_id  CHARACTER VARYING(255),
    instructions TEXT,
    extras JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_location_id ON location(id);

-- A trigger to update `updated_at` on row updates
CREATE OR REPLACE FUNCTION update_location_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER set_location_timestamp
BEFORE UPDATE ON location
FOR EACH ROW
EXECUTE FUNCTION update_location_timestamp();
