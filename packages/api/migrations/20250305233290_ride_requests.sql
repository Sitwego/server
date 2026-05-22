CREATE TYPE ride_request_status AS ENUM (
    'New',    -- The ride request has been made but not yet accepted
    'Accepted',   -- A driver has accepted the ride request
    'Inprogress',    -- The ride is in progress
    'Completed',  -- The ride has been successfully completed
    'Canceled',   -- The ride request was canceled by the rider or driver
    'Expired',    -- The ride request was not accepted within the allowed time
    'Failed',      -- The ride failed due to some issue (e.g., payment failure)
    'Arrived',
    'Waitingforrider'
);

CREATE TABLE ride_requests(
  id VARCHAR(26) PRIMARY KEY NOT NULL,
  driver_id VARCHAR(26) NOT NULL,
  customer_id VARCHAR(26) NOT NULL,
  fare NUMERIC(20, 2) NOT NULL,
  request_status ride_request_status NOT NULL DEFAULT 'New',
  estimated_distance_to_pickup DOUBLE PRECISION,
  estimated_duration_to_pickup INTEGER,
  estimated_distance DOUBLE PRECISION,
  search_request_valid_till TIMESTAMP WITH TIME ZONE,
  start_time TIMESTAMP WITH TIME ZONE,
  from_location_id VARCHAR(26) NOT NULL,
  to_location_id VARCHAR(26) NOT NULL,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL,
  updated_at TIMESTAMP WITH TIME ZONE NOT NULL,
  message TEXT,

  CONSTRAINT fk_from_location_id FOREIGN KEY (from_location_id) REFERENCES location(id) ON DELETE CASCADE,
  CONSTRAINT fk_to_location_id FOREIGN KEY (to_location_id) REFERENCES location(id) ON DELETE CASCADE
);

CREATE INDEX idx_ride_request ON ride_requests(id);
CREATE INDEX idx_ride_request_driver_id ON ride_requests(driver_id);