CREATE TABLE rating (
  id VARCHAR(26) PRIMARY KEY NOT NULL,          
  ride_id VARCHAR(26) NOT NULL,       
  driver_id VARCHAR(26) NOT NULL,
  rating_value INTEGER NOT NULL,
  customer_id VARCHAR(26) NOT NULL,
  feedback_details TEXT,
  was_offered_assistance BOOLEAN,
  attachment_id VARCHAR(26),         
  created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
  
  FOREIGN KEY (driver_id) REFERENCES profile(id),
  FOREIGN KEY (customer_id) REFERENCES profile(id),
  CONSTRAINT fk_ride_id FOREIGN KEY (ride_id) REFERENCES ride(id)
);

CREATE INDEX idx_rating_driver_id ON rating(driver_id);
CREATE INDEX idx_rating_ride_id ON rating(ride_id);