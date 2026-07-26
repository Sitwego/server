-- Step 7: order lifecycle state for status/track/cancel + unsolicited
-- on_status. All protocol state stays on beckn_orders (Rule #6) — these
-- columns live on the `confirm` row of an order:
--   fulfillment_state — network FulfillmentState code (NEW, RIDE_ASSIGNED,
--                       RIDE_ARRIVED_PICKUP, RIDE_STARTED, RIDE_ENDED,
--                       RIDE_CANCELLED)
--   assigned_json     — driver/vehicle/OTP snapshot captured when dispatch
--                       assigns a driver (serialized AssignedInfo)
--   pickup_gps/dropoff_gps — the trip as confirmed ("lat, lon"), so later
--                       on_status orders can repeat the stops.
ALTER TABLE beckn_orders
    ADD COLUMN fulfillment_state VARCHAR,
    ADD COLUMN assigned_json TEXT,
    ADD COLUMN pickup_gps VARCHAR(64),
    ADD COLUMN dropoff_gps VARCHAR(64);
