CREATE TYPE auto_pay_status AS ENUM (
  'enabled',
  'disabled',
  'pending_activation',
  'failed',
  'suspended',
  'cancelled',
  'not_set'
);
CREATE TABLE plans (
    id VARCHAR(26) PRIMARY KEY NOT NULL,
    vehicle_type VARCHAR(20) NOT NULL CHECK (vehicle_type IN ('TukTuk', 'Bike', 'Taxi')),
    plan_name VARCHAR(50) NOT NULL CHECK (plan_name IN ('Daily Unlimited', 'Daily Per Ride')),
    cost DECIMAL(10,2) NOT NULL,
    billing_type VARCHAR(10) NOT NULL CHECK (billing_type IN ('Per Day', 'Per Ride')),
    max_charge DECIMAL(10,2),
    max_rides INT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    no_ride_no_charge BOOLEAN DEFAULT FALSE
);

CREATE TABLE payment_authorizations (
    id VARCHAR(26) PRIMARY KEY NOT NULL,
    setup_date TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE subscriptions (
    id VARCHAR(26) PRIMARY KEY NOT NULL,
    driver_id VARCHAR(26) UNIQUE REFERENCES profile(id) ON DELETE CASCADE,
    plan_id VARCHAR(26) REFERENCES plans(id),
    payment_auth_id VARCHAR(26) REFERENCES payment_authorizations(id) ON DELETE CASCADE,
    payment_auth_setup_date TIMESTAMP WITH TIME ZONE,
    plan_end_date TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    auto_pay_status auto_pay_status NOT NULL DEFAULT 'not_set',
    service_name TEXT DEFAULT 'SUBSCRIPTION',
    last_payment_link_sent_at TIMESTAMP WITH TIME ZONE, -- ✅ Fixed
    enable_service_usage_charge BOOLEAN DEFAULT TRUE,
    is_on_free_trial BOOLEAN DEFAULT TRUE,
    is_category_level_subscription_enabled BOOLEAN
);

-- Indexes for better query performance
CREATE INDEX idx_driver_id ON subscriptions(driver_id);
CREATE INDEX idx_payment_auth_id ON subscriptions(payment_auth_id);
CREATE INDEX idx_service_name ON subscriptions(service_name);
CREATE INDEX idx_plan_id ON subscriptions(plan_id);
CREATE INDEX idx_created_at ON subscriptions USING btree (created_at DESC);

-- Bike: Daily Unlimited
INSERT INTO plans (id, vehicle_type, plan_name, cost, billing_type, no_ride_no_charge)
VALUES ('plan_bike_unlimited', 'Bike', 'Daily Unlimited', 40.00, 'Per Day', TRUE);

-- Bike: Daily Per Ride
INSERT INTO plans (id, vehicle_type, plan_name, cost, billing_type, max_charge, max_rides, no_ride_no_charge)
VALUES ('plan_bike_per_ride', 'Bike', 'Daily Per Ride', 6.00, 'Per Ride', 60.00, 10, FALSE);

-- Auto-TukTuk: Daily Unlimited
INSERT INTO plans (id, vehicle_type, plan_name, cost, billing_type, no_ride_no_charge)
VALUES ('plan_tuk_tuk_unlimited', 'TukTuk', 'Daily Unlimited', 60.00, 'Per Day', TRUE);

-- Auto-TukTuk: Daily Per Ride
INSERT INTO plans (id, vehicle_type, plan_name, cost, billing_type, max_charge, max_rides, no_ride_no_charge)
VALUES ('plan_tuk_tuk_per_ride', 'TukTuk', 'Daily Per Ride', 10.00, 'Per Ride', 100.00, 10, FALSE);

-- Taxi: Daily Unlimited
INSERT INTO plans (id, vehicle_type, plan_name, cost, billing_type, no_ride_no_charge)
VALUES ('plan_taxi_unlimited', 'Taxi', 'Daily Unlimited', 125.00, 'Per Day', TRUE);

-- Taxi: Daily Per Ride
INSERT INTO plans (id, vehicle_type, plan_name, cost, billing_type, max_charge, max_rides, no_ride_no_charge)
VALUES ('plan_taxi_per_ride', 'Taxi', 'Daily Per Ride', 20.00, 'Per Ride', 200.00, 10, FALSE);

