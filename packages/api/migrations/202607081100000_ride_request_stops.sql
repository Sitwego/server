-- Intermediate stops for a ride request (Pickup → Stop(s) → DropOff).
-- One row per stop; stop_order is the 0-based visit sequence. The API layer
-- currently caps rides at one stop (MAX_RIDE_STOPS in rides.rs) — lifting
-- that cap needs no schema change here.
CREATE TABLE ride_request_stops (
    ride_request_id VARCHAR(26) NOT NULL REFERENCES ride_requests(id) ON DELETE CASCADE,
    stop_order INTEGER NOT NULL,
    location_id VARCHAR(26) NOT NULL REFERENCES location(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (ride_request_id, stop_order)
);

CREATE INDEX idx_ride_request_stops_location_id ON ride_request_stops(location_id);
