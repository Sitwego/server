CREATE TABLE "ipn" (
  id VARCHAR(26) NOT NULL PRIMARY KEY UNIQUE,
  checkout_request_id VARCHAR(255) NOT NULL UNIQUE,
  merchant_request_id VARCHAR(255) NOT NULL UNIQUE,
  amount DECIMAL(10,2) NOT NULL DEFAULT 0.00,
  mpesa_receipt_number VARCHAR(50),  -- M-Pesa receipt (e.g., 'TI74VBMJ92')
  currency VARCHAR(3) NOT NULL DEFAULT 'KES',
  payment_status VARCHAR(20) NOT NULL DEFAULT 'pending',
  payment_method VARCHAR(20) NOT NULL DEFAULT 'mpesa',
  created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  transaction_date BIGINT NOT NULL,  -- i64 Unix timestamp (e.g., 20250907203605)
  phone_number VARCHAR(15),  -- Optional, for customer phone (e.g., '254700000000')
  result_desc TEXT  -- Optional description (e.g., 'The service request is processed successfully.')
);

-- Indexes for performance (query by ID, request IDs, or date)
CREATE INDEX idx_ipn_checkout_request_id ON "ipn" (checkout_request_id);
CREATE INDEX idx_ipn_merchant_request_id ON "ipn" (merchant_request_id);
CREATE INDEX idx_ipn_transaction_date ON "ipn" (transaction_date);
CREATE INDEX idx_ipn_payment_status ON "ipn" (payment_status);
CREATE INDEX idx_ipn_phone_number ON "ipn" (phone_number);
CREATE INDEX idx_ipn_mpesa_receipt_number ON "ipn" (mpesa_receipt_number);

-- Trigger to auto-update updated_at on row changes
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_ipn_updated_at BEFORE UPDATE ON "ipn"
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();