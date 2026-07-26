-- Beckn confirm is confirm-then-assign (decision B): the ride id is minted at
-- `confirm` as the dispatch request id, and the `ride` row only comes into
-- existence later when a driver accepts and the trip starts (start_ride reuses
-- ride_request.id as ride.id). The FK therefore cannot hold at confirm time —
-- keep ride_id as a soft reference; the index from the original migration stays.
ALTER TABLE beckn_orders DROP CONSTRAINT beckn_orders_ride_id_fkey;
