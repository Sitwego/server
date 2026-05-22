CREATE TABLE "profile" (
    id VARCHAR(26) NOT NULL PRIMARY KEY,
    nonce BYTEA NOT NULL,
    contact_data BYTEA NOT NULL,
    first_name VARCHAR(255) NOT NULL,
    middle_name VARCHAR(255),
    last_name VARCHAR(255) NOT NULL,
    gender VARCHAR(255) NOT NULL,
    hometown VARCHAR(255),
    mobile_country_code VARCHAR(255),
    identifier VARCHAR(255),
    is_new BOOLEAN DEFAULT TRUE,
    verified BOOLEAN NOT NULL DEFAULT FALSE,
    device_token VARCHAR(255),
    whatsapp_notification_status VARCHAR(255),
    face_image_id VARCHAR,
    total_earned_coins INTEGER DEFAULT 0,
    used_coins INTEGER DEFAULT 0,
    registration_lat DOUBLE PRECISION,
    registration_lon DOUBLE PRECISION,
    client_device_type VARCHAR(255),
    client_device_id VARCHAR(255),
    backend_app_version VARCHAR(255),
    driver_tag TEXT[],
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
-- Indexes for faster lookups
CREATE INDEX profile_idx ON profile (id, created_at DESC);
CREATE INDEX idx_profile_identifier ON profile (identifier);

CREATE TABLE "driver" (
    id VARCHAR PRIMARY KEY REFERENCES profile(id) ON DELETE CASCADE,
    password VARCHAR(255) NOT NULL,
    email_hash VARCHAR NOT NULL,
    phone_hash VARCHAR NOT NULL,
    photo_id VARCHAR(255),
    photo_nonce BYTEA,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
CREATE INDEX drivers_idx ON driver (id, created_at DESC);
CREATE INDEX idx_driver_email_hash ON driver (email_hash);
CREATE INDEX idx_driver_phone_hash ON driver (phone_hash);

CREATE TABLE "customer" (
    id VARCHAR PRIMARY KEY REFERENCES profile(id) ON DELETE CASCADE,
    password VARCHAR(255) NOT NULL,
    email_hash VARCHAR NOT NULL,
    phone_hash VARCHAR NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
CREATE INDEX customer_idx ON customer (id, created_at DESC);
CREATE INDEX idx_customer_email_hash ON customer (email_hash);
CREATE INDEX idx_customer_phone_hash ON customer (phone_hash);

-- CREATE TABLE "driver_doc" (
--     id VARCHAR NOT NULL PRIMARY KEY UNIQUE,
--     e_data BYTEA NOT NULL,
--     nonce BYTEA NOT NULL,
--     created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
--     updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
--     -- passport_photo VARCHAR NOT NULL,
--     -- national_id VARCHAR NOT NULL,
--     -- certificate_of_good_conduct VARCHAR NOT NULL,
--     -- drivers_license VARCHAR NOT NULL,
--     -- psv_license VARCHAR NOT NULL,
--     -- vehicle_logbook_or_sales_agreement VARCHAR NOT NULL,
--     -- vehicle_inspection_sticker VARCHAR NOT NULL,
--     -- psv_insurance_sticker VARCHAR NOT NULL
-- );
-- CREATE INDEX driver_docs_idx ON driver_doc (id);

CREATE TYPE vehicle_category AS ENUM (
    'Swift',
    'Standard',
    'Comfort',
    'Bike',
    'Women',
    'Xl',
    'Executive'
);

CREATE TABLE vehicle_categories (
    category vehicle_category PRIMARY KEY,
    engine_size VARCHAR(50),
    example_cars VARCHAR(100),
    short_distance_kes_per_km INTEGER,
    long_distance_kes_per_km INTEGER,
    base_fare_kes INTEGER,
    waiting_per_minute_kes INTEGER,
    return_trip_discount VARCHAR(10),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL
);

-- Additional indexes for faster queries
CREATE INDEX idx_engine_size ON vehicle_categories (engine_size);
CREATE INDEX idx_short_distance_price ON vehicle_categories (short_distance_kes_per_km);
CREATE INDEX idx_long_distance_price ON vehicle_categories (long_distance_kes_per_km);
CREATE INDEX idx_base_fare ON vehicle_categories (base_fare_kes);

CREATE TABLE "vehicle" (
    id VARCHAR(26) NOT NULL PRIMARY KEY UNIQUE,
    color VARCHAR(20) NOT NULL,
    vehicle_type VARCHAR NOT NULL,
    plate_number VARCHAR(20) NOT NULL,
    capacity INTEGER,
    model VARCHAR(100),
    y_manufacturing INTEGER,
    make VARCHAR(100),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
CREATE INDEX vehicle_idx ON vehicle (id);

CREATE TABLE "driver_earning" (
    id VARCHAR NOT NULL PRIMARY KEY UNIQUE,
    driver_id VARCHAR(26) NOT NULL REFERENCES driver(id) ON DELETE CASCADE,
    amount DECIMAL(10,2) NOT NULL,
    is_discounted BOOLEAN NOT NULL DEFAULT FALSE,
    discount DECIMAL(10,2) NOT NULL DEFAULT 0.00,
    currency VARCHAR(3) NOT NULL DEFAULT 'KES', 
    payment_status VARCHAR(20) NOT NULL DEFAULT 'pending',
    payment_method VARCHAR(50) NOT NULL DEFAULT 'cash',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX driver_earnings_idx ON driver_earning (driver_id, created_at);

CREATE INDEX idx_driver_earnings_driver ON driver_earning (driver_id, created_at DESC);

CREATE INDEX idx_driver_earnings_status ON driver_earning (payment_status);

