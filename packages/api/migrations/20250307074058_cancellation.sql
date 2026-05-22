-- Cancellation Reasons Table
CREATE TABLE cancellation_reason (
    id VARCHAR(26) PRIMARY KEY NOT NULL,
    reason_code TEXT UNIQUE NOT NULL,  
    description TEXT NOT NULL,
    enabled BOOLEAN DEFAULT TRUE,
    priority INT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Ride Cancellations Table
CREATE TABLE ride_cancellations (
    id VARCHAR(26) PRIMARY KEY NOT NULL,
    ride_id VARCHAR(26) REFERENCES ride(id) ON DELETE SET NULL,
    canceled_by VARCHAR(26) REFERENCES profile(id) ON DELETE SET NULL,
    reason_code TEXT REFERENCES cancellation_reason(reason_code) ON DELETE SET NULL,
    cancellation_time TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for better query performance
CREATE INDEX idx_enabled ON cancellation_reason(enabled);
CREATE INDEX idx_priority ON cancellation_reason(priority);
CREATE INDEX idx_cancellation_ride_id ON ride_cancellations(ride_id);
CREATE INDEX idx_canceled_by ON ride_cancellations(canceled_by);
CREATE INDEX idx_reason_code ON ride_cancellations(reason_code);
CREATE INDEX idx_cancellation_time ON ride_cancellations USING btree (cancellation_time DESC);

-- -- Sample Data
-- INSERT INTO cancellation_reason (id, reason_code, description, enabled, priority)
-- VALUES
--     (gen_random_uuid(), 'RIDER_NO_SHOW', 'Rider did not show up', TRUE, 1),
--     (gen_random_uuid(), 'DRIVER_CANCELLED', 'Driver canceled the ride', TRUE, 2),
--     (gen_random_uuid(), 'FARE_TOO_HIGH', 'Fare was too high for the rider', TRUE, 3);

-- INSERT INTO ride_cancellations (id, ride_id, canceled_by, reason_code)
-- VALUES
--     (gen_random_uuid(), 'ride123', 'user456', 'RIDER_NO_SHOW'),
--     (gen_random_uuid(), 'ride789', 'user321', 'DRIVER_CANCELLED');
