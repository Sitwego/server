-- Add the 'Auto' vehicle category for three-wheeler auto-rickshaws (tuktuks).
-- Auto is an exclusive category like Bike and Women: an Auto driver serves only
-- Auto requests, and no other category driver serves Auto requests. Eligibility
-- is enforced in code (VehicleCategory::eligible_serving_categories).
ALTER TYPE vehicle_category ADD VALUE IF NOT EXISTS 'Auto';
