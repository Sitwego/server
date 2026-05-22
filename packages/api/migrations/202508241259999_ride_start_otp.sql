ALTER TABLE ride_requests ADD COLUMN otp VARCHAR(6);
ALTER TABLE ride_requests ADD COLUMN otp_verified BOOLEAN DEFAULT FALSE;
-- Add also ride_end_otp
ALTER TABLE ride_requests ADD COLUMN end_otp VARCHAR(6);
ALTER TABLE ride_requests ADD COLUMN end_otp_verified BOOLEAN DEFAULT FALSE;
-- Add index on otp for faster lookup
CREATE INDEX idx_ride_request_otp ON ride_requests(otp);
CREATE INDEX idx_ride_request_end_otp ON ride_requests(end_otp);

