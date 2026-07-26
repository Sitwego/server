-- Persist why a ride request was cancelled alongside the status flip, so
-- cancellations survive as auditable rows instead of being deleted.
-- canceled_by records which side ended it: 'driver' | 'customer' | 'admin'.
ALTER TABLE ride_requests
    ADD COLUMN cancel_reason TEXT,
    ADD COLUMN cancel_note TEXT,
    ADD COLUMN canceled_by TEXT;

-- Partial: canceled_by is NULL on every non-cancelled row, so indexing only
-- the cancelled subset keeps this small while serving per-side lookups.
CREATE INDEX idx_ride_requests_canceled_by
    ON ride_requests(canceled_by)
    WHERE canceled_by IS NOT NULL;
