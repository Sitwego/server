
ALTER TABLE ride_requests
  DROP COLUMN IF EXISTS from_latitude,
  DROP COLUMN IF EXISTS from_longitude,
  DROP COLUMN IF EXISTS to_latitude,
  DROP COLUMN IF EXISTS to_longitude;

ALTER TABLE ride_requests 
  ADD COLUMN from_latitude DOUBLE PRECISION,
  ADD COLUMN from_longitude DOUBLE PRECISION,
  ADD COLUMN to_latitude DOUBLE PRECISION,
  ADD COLUMN to_longitude DOUBLE PRECISION;