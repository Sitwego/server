-- Step 6 (confirm): persist the quote agreed at init so confirm can honour it
-- without re-pricing. Protocol state lives on beckn_orders, never on rides
-- (adapter Rule #6).
ALTER TABLE beckn_orders
    ADD COLUMN quoted_fare INTEGER,
    ADD COLUMN quoted_item_id VARCHAR(64);
