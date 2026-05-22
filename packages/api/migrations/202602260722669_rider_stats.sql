CREATE TABLE rider_stats (
    customer_id VARCHAR PRIMARY KEY NOT NULL,
    total_rides INTEGER NOT NULL DEFAULT 0,
    total_spent DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_distance DOUBLE PRECISION NOT NULL DEFAULT 0,
    rides_cancelled INTEGER NOT NULL DEFAULT 0,
    total_coins_earned INTEGER NOT NULL DEFAULT 0,
    total_coins_spent INTEGER NOT NULL DEFAULT 0,
    rating DECIMAL(3,1) NOT NULL DEFAULT 0.0,
    total_ratings INTEGER NOT NULL DEFAULT 0,
    total_rating_score DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    is_valid_rating BOOLEAN,
    fav_driver_count INTEGER NOT NULL DEFAULT 0,
    total_referral_counts INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    FOREIGN KEY (customer_id) REFERENCES customer(id) ON DELETE CASCADE
);

CREATE INDEX idx_rider_stats_customer_id ON rider_stats(customer_id);
CREATE INDEX idx_rider_stats_updated_at ON rider_stats(updated_at DESC);
