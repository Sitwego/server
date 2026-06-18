ALTER TABLE driver_documents
    ALTER COLUMN reviewed_by TYPE TEXT;

ALTER TABLE driver_identity_documents
    ALTER COLUMN reviewed_by TYPE TEXT;

ALTER TABLE driver
    ALTER COLUMN photo_reviewed_by TYPE TEXT;
