-- Backfill the accrual watermark to deploy time so the job starts billing from
-- here forward. Without this, the watermark would bootstrap off the stale
-- `updated_at` and bill every historical ride at once on the first run.
-- Kept in a separate migration from the ADD COLUMN so the already-applied
-- column migration's checksum is not disturbed.
UPDATE subscriptions SET last_accrued_at = now() WHERE last_accrued_at IS NULL;
