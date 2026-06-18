-- The driver's profile photo (selfie) is an onboarding artifact that must be
-- verified like the uploaded documents. It lives on the `driver` row (one photo
-- per driver), so its review state goes here rather than in a documents table.
-- Reuses the existing document_review_status enum. Existing drivers default to
-- PENDING; a driver with no photo simply won't surface a photo card.
ALTER TABLE driver
    ADD COLUMN photo_review_status document_review_status NOT NULL DEFAULT 'PENDING',
    ADD COLUMN photo_reviewed_by   VARCHAR(26),
    ADD COLUMN photo_reviewed_at   TIMESTAMP WITH TIME ZONE,
    ADD COLUMN photo_reject_reason TEXT;
