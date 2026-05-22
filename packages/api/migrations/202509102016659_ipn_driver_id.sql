ALTER TABLE ipn ADD COLUMN driver_id VARCHAR(26) NOT NULL REFERENCES driver(id);
CREATE INDEX idx_ipn_driver_id ON ipn (driver_id);