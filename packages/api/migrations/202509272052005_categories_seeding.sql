-- Insert the data
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
    ('Swift', '650cc – 1050cc', 'Suzuki Alto, Daihatsu Mira,Toyota Vitz, Nissan March,', 33.10, 35.10, 150, 7, '10%', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, 260, 4.6),
    ('Standard', '1051cc – 1300cc', 'Toyota Axio, Nissan Tiida, Honda Fit, Mazda Demio', 36.40, 40, 250, 10, '19%', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, 300, 5),
    ('Comfort', '1301cc – 1500cc', 'Toyota Premio, Honda Civic', 40.33, 50, 330, 12, '20%', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, 350, 5),
    ('Xl', '7-seaters, SUVs', 'Toyota Noah, Nissan Serena', 51.60, 60, 420, 15, '20%', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, 420, 8),
    ('Executive', 'Luxury Sedans, SUVs', 'Mercedes E-Class, BMW 5 Series', 70, 80, 520, 20, '20%', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, 500, 10);