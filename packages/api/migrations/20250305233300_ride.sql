CREATE TABLE ride (
    id VARCHAR(26) NOT NULL PRIMARY KEY REFERENCES ride_requests(id) ON DELETE CASCADE,
    customer_id VARCHAR(26) NOT NULL,
    status character varying(255) NOT NULL,
    driver_id VARCHAR(26) NOT NULL,
    otp character(4) NOT NULL,
    end_otp text,
    tracking_url character varying(255) NOT NULL,
    fare numeric,
    currency text,
    traveled_distance double precision NOT NULL DEFAULT 0,
    chargeable_distance integer,
    distance_unit text,
    driver_arrival_time timestamp with time zone,
    trip_start_time timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    trip_end_time timestamp with time zone,
    trip_start_lat double precision,
    trip_start_lon double precision,
    trip_end_lat double precision,
    trip_end_lon double precision,
    pickup_drop_outside_of_threshold boolean,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    driver_deviated_to_toll_route boolean,
    driver_deviated_from_route boolean,
    number_of_snap_to_road_calls integer,
    number_of_osrm_snap_to_road_calls integer,
    number_of_self_tuned integer,
    number_of_deviation boolean,
    ui_distance_calculation_with_accuracy integer,
    ui_distance_calculation_without_accuracy integer,
    is_free_ride boolean,
    estimated_toll_charges numeric,
    estimated_toll_names text[],
    safety_alert_triggered boolean NOT NULL DEFAULT false,
    enable_frequent_location_updates boolean NOT NULL DEFAULT false,
    ride_ended_by VARCHAR(26),
    trip_category text,
    online_payment boolean NOT NULL DEFAULT false,
    cancellation_fee_if_cancelled numeric,
    tip_amount numeric,
    ride_tags text[],
    ride_type text,
    FOREIGN KEY (customer_id) REFERENCES profile(id),
    FOREIGN KEY (driver_id) REFERENCES profile(id),
    FOREIGN KEY (ride_ended_by) REFERENCES profile(id)
    -- FOREIGN KEY (trip_category) REFERENCES trip_category(id)
);

-- Create indexes for secondary keys
CREATE INDEX idx_ride_driver_id ON ride(driver_id);

ALTER TABLE driver_earning ADD COLUMN ride_id VARCHAR NOT NULL UNIQUE REFERENCES ride(id) ON DELETE CASCADE;
CREATE INDEX idx_driver_earnings_ride ON driver_earning (ride_id);