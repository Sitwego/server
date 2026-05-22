CREATE TYPE driver_document_type AS ENUM (
  'DRIVING_LICENSE',
  'PSV_BADGE',
  'PSV_INSURANCE',
  'CERTIFICATE_OF_GOOD_CONDUCT',
  'VEHICLE_INSPECTION_STICKER',
  'KRA',
  'NONE'
);

CREATE TABLE driver_documents (
    id BIGSERIAL PRIMARY KEY,
    driver_id VARCHAR(26) NOT NULL REFERENCES driver(id) ON DELETE CASCADE,
    document_type driver_document_type NOT NULL,
    version INT NOT NULL DEFAULT 1,
    file_id VARCHAR(255) NOT NULL,
    nonce BYTEA NOT NULL,
    metadata JSONB DEFAULT '{}'::jsonb,    -- for extra fields like expiry_date
    is_active BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE driver_identity_documents (
    id BIGSERIAL PRIMARY KEY,
    driver_id VARCHAR(26) NOT NULL REFERENCES driver(id) ON DELETE CASCADE,
    id_number TEXT NOT NULL,
    document_subtype TEXT NOT NULL CHECK (document_subtype IN ('national_id', 'passport')),
    file_id_front VARCHAR(255) NOT NULL,
    front_nonce BYTEA NOT NULL,
    file_id_back VARCHAR(255) NOT NULL,
    back_nonce BYTEA NOT NULL,
    version INT NOT NULL DEFAULT 1,
    is_active BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE UNIQUE INDEX unique_active_doc_idx ON driver_documents(driver_id, document_type) WHERE is_active = TRUE;

CREATE UNIQUE INDEX unique_active_identity_doc_idx ON driver_identity_documents(driver_id, document_subtype, is_active) WHERE (is_active = TRUE);

CREATE INDEX idx_driver_documents_driver_id ON driver_documents(driver_id);
CREATE INDEX idx_driver_identity_documents_driver_id ON driver_identity_documents(driver_id);
CREATE INDEX idx_identity_id_number ON driver_identity_documents(id_number);
CREATE INDEX idx_metadata_gin ON driver_documents USING GIN (metadata);



CREATE OR REPLACE FUNCTION set_document_version()
RETURNS TRIGGER AS $$
BEGIN
    SELECT COALESCE(MAX(version), 0) + 1 INTO NEW.version
    FROM driver_documents
    WHERE driver_id = NEW.driver_id AND document_type = NEW.document_type;

    UPDATE driver_documents
    SET is_active = FALSE
    WHERE driver_id = NEW.driver_id AND document_type = NEW.document_type AND is_active = TRUE;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER auto_version_before_insert
BEFORE INSERT ON driver_documents
FOR EACH ROW
EXECUTE FUNCTION set_document_version();

CREATE OR REPLACE FUNCTION set_identity_doc_version()
RETURNS TRIGGER AS $$
BEGIN
    SELECT COALESCE(MAX(version), 0) + 1 INTO NEW.version
    FROM driver_identity_documents
    WHERE driver_id = NEW.driver_id AND document_subtype = NEW.document_subtype;

    UPDATE driver_identity_documents
    SET is_active = FALSE
    WHERE driver_id = NEW.driver_id AND document_subtype = NEW.document_subtype AND is_active = TRUE;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER auto_version_identity
BEFORE INSERT ON driver_identity_documents
FOR EACH ROW
EXECUTE FUNCTION set_identity_doc_version();


