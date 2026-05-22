DROP TABLE IF EXISTS rating;

-- Customer rates the driver
CREATE TABLE driver_rating (
  id VARCHAR(26) PRIMARY KEY NOT NULL,
  ride_id VARCHAR(26) NOT NULL,
  driver_id VARCHAR(26) NOT NULL,
  customer_id VARCHAR(26) NOT NULL,
  rating_value INTEGER NOT NULL,
  punctuality INTEGER,
  driving_behavior INTEGER,
  safety_compliance INTEGER,
  vehicle_cleanliness INTEGER,
  feedback_details TEXT,
  was_offered_assistance BOOLEAN,
  attachment_id VARCHAR(26),
  created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

  FOREIGN KEY (driver_id) REFERENCES driver(id),
  FOREIGN KEY (customer_id) REFERENCES customer(id),
  CONSTRAINT fk_driver_rating_ride_id FOREIGN KEY (ride_id) REFERENCES ride(id),
  CONSTRAINT uq_driver_rating_per_ride UNIQUE (ride_id, customer_id)
);

CREATE INDEX idx_driver_rating_driver_id ON driver_rating(driver_id);
CREATE INDEX idx_driver_rating_ride_id ON driver_rating(ride_id);

-- Driver rates the customer
CREATE TABLE customer_rating (
  id VARCHAR(26) PRIMARY KEY NOT NULL,
  ride_id VARCHAR(26) NOT NULL,
  customer_id VARCHAR(26) NOT NULL,
  driver_id VARCHAR(26) NOT NULL,
  rating_value INTEGER NOT NULL,
  punctuality INTEGER,
  respectfulness INTEGER,
  fare_readiness INTEGER,
  feedback_details TEXT,
  attachment_id VARCHAR(26),
  created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

  FOREIGN KEY (customer_id) REFERENCES customer(id),
  FOREIGN KEY (driver_id) REFERENCES driver(id),
  CONSTRAINT fk_customer_rating_ride_id FOREIGN KEY (ride_id) REFERENCES ride(id),
  CONSTRAINT uq_customer_rating_per_ride UNIQUE (ride_id, driver_id)
);

CREATE INDEX idx_customer_rating_customer_id ON customer_rating(customer_id);
CREATE INDEX idx_customer_rating_ride_id ON customer_rating(ride_id);
