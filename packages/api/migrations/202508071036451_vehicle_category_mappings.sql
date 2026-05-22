ALTER TABLE vehicle ADD COLUMN vin VARCHAR(255);
-- Junction table to associate vehicles with multiple categories
CREATE TABLE vehicle_category_mappings (
    vehicle_id VARCHAR REFERENCES vehicle(id) ON DELETE CASCADE,
    category vehicle_category REFERENCES vehicle_categories(category) ON DELETE CASCADE,
    driver_id VARCHAR(26) REFERENCES driver(id) ON DELETE CASCADE,
    PRIMARY KEY (vehicle_id, category, driver_id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Table to define category qualification rules (e.g., Standard qualifies for Swift)
CREATE TABLE category_qualifications (
    primary_category vehicle_category REFERENCES vehicle_categories(category) ON DELETE CASCADE,
    qualified_category vehicle_category REFERENCES vehicle_categories(category) ON DELETE CASCADE,
    PRIMARY KEY (primary_category, qualified_category),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);