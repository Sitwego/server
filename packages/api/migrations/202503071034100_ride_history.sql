CREATE EXTENSION IF NOT EXISTS postgis;

CREATE TABLE ride_history (
    ride_id VARCHAR(26) PRIMARY KEY NOT NULL,
    driver_id VARCHAR(26) NOT NULL,
    coordinates GEOGRAPHY(LINESTRING, 4326),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    FOREIGN KEY (ride_id) REFERENCES ride(id) ON DELETE CASCADE,
    FOREIGN KEY (driver_id) REFERENCES profile(id)
);
-- Index for spatial queries
CREATE INDEX idx_ride_history_coordinates ON ride_history USING GIST (coordinates);