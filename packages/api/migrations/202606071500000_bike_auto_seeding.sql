-- Seed the Bike and Auto (three-wheeler auto-rickshaw) vehicle categories.
-- The original seeding migration only covered Swift–Executive.
--   Bike: base fare 70, 15/km short & 20/km long, 5/min waiting,
--         5% return-trip discount, min fare 100, 2 per-min rate.
--   Auto: base fare 70, 20/km short & 25/km long, 5/min waiting,
--         10% return-trip discount, min fare 120, 2.5 per-min rate.
INSERT INTO vehicle_categories (
    category,
    engine_size,
    example_cars,
    short_distance_kes_per_km,
    long_distance_kes_per_km,
    base_fare_kes,
    waiting_per_minute_kes,
    return_trip_discount,
    created_at,
    updated_at,
    min_fare,
    per_min_rate
) VALUES
    ('Bike', '100cc+', 'Boda Boda', 15, 20, 70, 5, '5%', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, 100, 2),
    ('Auto', '150cc+', 'Tuktuk, Bajaj', 20, 25, 70, 5, '10%', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, 120, 2.5)
ON CONFLICT (category) DO NOTHING;
