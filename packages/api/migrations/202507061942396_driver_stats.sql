CREATE TABLE driver_stats (
    driver_id VARCHAR(26) PRIMARY KEY NOT NULL,
    idle_since TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    total_rides INTEGER NOT NULL DEFAULT 0,
    total_earnings DOUBLE PRECISION NOT NULL DEFAULT 0,
    bonus_earned DOUBLE PRECISION NOT NULL DEFAULT 0,
    late_night_trips INTEGER NOT NULL DEFAULT 0,
    earnings_missed DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_distance DOUBLE PRECISION NOT NULL DEFAULT 0,
    rides_cancelled INTEGER NOT NULL DEFAULT 0,
    total_rides_assigned INTEGER NOT NULL DEFAULT 0,
    coin_coverted_to_cash_left DECIMAL(15,4),
    total_coins_converted_cash DECIMAL(15,4),
    distance_unit VARCHAR(20),
    rating DECIMAL(3,1) NOT NULL DEFAULT 0.0,
    total_ratings INTEGER NOT NULL DEFAULT 0,
    total_rating_score DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    is_valid_rating BOOLEAN,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    fav_rider_count INTEGER NOT NULL DEFAULT 0,
    total_payout_earnings DECIMAL(15,4),
    total_valid_activated_rides INTEGER,
    total_referral_counts INTEGER NOT NULL DEFAULT 0,
    total_payout_amount_paid DECIMAL(15,4),
    
    FOREIGN KEY (driver_id) REFERENCES driver(id) ON DELETE CASCADE
);
CREATE INDEX idx_driver_stats_driver_id ON driver_stats(driver_id);
CREATE INDEX idx_driver_stats_updated_at ON driver_stats(updated_at DESC);