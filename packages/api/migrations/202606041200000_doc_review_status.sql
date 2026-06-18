-- Admin document verification: each uploaded driver document is reviewed by an
-- admin before the driver can go live. We track the review decision, who made
-- it, when, and (on rejection) the reason shown to the driver so they can
-- re-upload. The activation flag on `driver` is flipped separately (hybrid:
-- admin confirms once all required docs are APPROVED), so this column does not
-- touch driver.activated.

CREATE TYPE document_review_status AS ENUM (
  'PENDING',
  'APPROVED',
  'REJECTED'
);

-- Typed docs (license, PSV badge/insurance, good conduct, inspection, KRA).
ALTER TABLE driver_documents
    ADD COLUMN review_status document_review_status NOT NULL DEFAULT 'PENDING',
    ADD COLUMN reviewed_by   VARCHAR(26),
    ADD COLUMN reviewed_at   TIMESTAMP WITH TIME ZONE,
    ADD COLUMN reject_reason TEXT;

-- Identity docs (national ID / passport, front + back).
ALTER TABLE driver_identity_documents
    ADD COLUMN review_status document_review_status NOT NULL DEFAULT 'PENDING',
    ADD COLUMN reviewed_by   VARCHAR(26),
    ADD COLUMN reviewed_at   TIMESTAMP WITH TIME ZONE,
    ADD COLUMN reject_reason TEXT;

-- Admin review queue: fetch the active docs still awaiting a decision.
CREATE INDEX idx_driver_documents_pending
    ON driver_documents(driver_id)
    WHERE is_active = TRUE AND review_status = 'PENDING';

CREATE INDEX idx_driver_identity_documents_pending
    ON driver_identity_documents(driver_id)
    WHERE is_active = TRUE AND review_status = 'PENDING';
