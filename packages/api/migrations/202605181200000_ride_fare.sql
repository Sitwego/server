CREATE TABLE ride_fare (
    id          VARCHAR(26) PRIMARY KEY,
    ride_id     VARCHAR(26) NOT NULL REFERENCES ride(id) ON DELETE CASCADE,

    -- flexible fare components: fare, waiting_charge, tolls, extra_dx, etc.
    components  JSONB NOT NULL DEFAULT '{}',

    -- stored explicitly so aggregation stays clean without parsing JSONB
    total       numeric NOT NULL DEFAULT 0,

    status      text NOT NULL, -- 'estimated' | 'adjusted' | 'final'
    reason      text,          -- e.g. 'toll route taken', 'driver waited 8min', 'dispute resolved'

    recorded_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_ride_fare_ride_id ON ride_fare(ride_id);

-- drop the flat fare column from ride now that ride_fare is the source of truth
-- ALTER TABLE ride DROP COLUMN IF EXISTS fare;
