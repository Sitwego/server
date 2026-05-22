-- Booking Table
CREATE TABLE booking (
    id VARCHAR(26) PRIMARY KEY NOT NULL,
    customer_id VARCHAR(26) NOT NULL,
    payment_method_id TEXT,
    payment_link TEXT,
    status CHARACTER VARYING(255) NOT NULL,
    driver_id VARCHAR(26) NOT NULL,
    is_booking_updated BOOLEAN NOT NULL DEFAULT FALSE,
    start_time TIMESTAMP WITH TIME ZONE NOT NULL,
    return_time TIMESTAMP WITH TIME ZONE,
    round_trip BOOLEAN,
    from_location_id VARCHAR(26) NOT NULL,
    to_location_id VARCHAR(26) NOT NULL,
    estimated_fare NUMERIC(30,2) NOT NULL,
    estimated_distance DOUBLE PRECISION,
    distance_unit CHARACTER VARYING(255),
    estimated_duration INTEGER,
    estimated_static_duration INTEGER,
    discount NUMERIC(30,2),
    estimated_total_fare NUMERIC(30,2) NOT NULL,
    is_scheduled BOOLEAN NOT NULL,
    -- Note: booking_details is handled as separate fields based on the union type
    fare_product_type CHARACTER VARYING(255) NOT NULL DEFAULT 'ONE_WAY',
    distance_value DOUBLE PRECISION,
    stop_location_id CHARACTER VARYING(36),
    otp_code CHARACTER(4),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL,
    service_tier_name TEXT,
    service_tier_short_desc TEXT,
    payment_status TEXT,
    trip_category TEXT,
    initiated_by TEXT,
    currency CHARACTER VARYING(255),
    estimated_distance_value DOUBLE PRECISION,
    estimated_duration_value INTEGER,

    CONSTRAINT fk_customer FOREIGN KEY (customer_id) REFERENCES profile(id),
    CONSTRAINT fk_driver FOREIGN KEY (driver_id) REFERENCES profile(id),
    CONSTRAINT fk_booking_from_location_id FOREIGN KEY (from_location_id) REFERENCES location(id) ON DELETE CASCADE,
    CONSTRAINT fk_booking_to_location_id FOREIGN KEY (to_location_id) REFERENCES location(id) ON DELETE CASCADE
);

-- Create indexes for secondary keys
CREATE INDEX idx_booking_customer_id ON booking(customer_id);
CREATE INDEX idx_booking_driver_id ON booking(driver_id);
CREATE INDEX idx_booking_from_location_id ON booking(from_location_id);

-- Booking Parties Link Table
CREATE TABLE booking_parties_link (
    id VARCHAR(26) PRIMARY KEY,
    booking_id CHARACTER(36) NOT NULL,
    party_id CHARACTER(36) NOT NULL,
    party_type TEXT NOT NULL,
    party_name TEXT NOT NULL,
    is_active BOOLEAN NOT NULL,
    FOREIGN KEY (booking_id) REFERENCES booking(id)
);

-- Create indexes for secondary keys in booking_parties_link
CREATE INDEX idx_booking_parties_link_party_id ON booking_parties_link(party_id);
CREATE INDEX idx_booking_parties_link_booking_id ON booking_parties_link(booking_id);
