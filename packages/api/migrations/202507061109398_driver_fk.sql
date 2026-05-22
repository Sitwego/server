ALTER TABLE vehicle ADD COLUMN driver_id VARCHAR(26) REFERENCES driver(id);
CREATE INDEX vehicle_driver_id_idx ON vehicle(driver_id);