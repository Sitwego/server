-- Envelope encryption: store the KMS ciphertext blob alongside each nonce so
-- the data key can be recovered after a restart via kms:Decrypt(blob).
-- Previously only the nonce was stored; the plaintext key was derived from the
-- KMS key ARN at runtime, which caused decryption failures after restarts
-- (generate_data_key produces a *new* random key on every call).

-- profile: contact_data is encrypted with a per-row data key whose ciphertext
-- blob is now stored here. Required for all existing and future rows.
ALTER TABLE profile
    ADD COLUMN encrypted_key BYTEA NOT NULL DEFAULT '\x'::bytea;

-- driver: photo is uploaded to S3 with envelope encryption; the ciphertext blob
-- must be stored so the photo can be decrypted after a restart. Nullable because
-- drivers without a photo have no blob to store.
ALTER TABLE driver
    ADD COLUMN photo_encrypted_key BYTEA;

-- driver_documents: each document file uploaded to S3 is encrypted with a
-- per-file data key. The ciphertext blob is required for decryption.
ALTER TABLE driver_documents
    ADD COLUMN encrypted_key BYTEA NOT NULL DEFAULT '\x'::bytea;

-- driver_identity_documents: front and back images are each encrypted with
-- their own data key. Both blobs are required for decryption.
ALTER TABLE driver_identity_documents
    ADD COLUMN front_encrypted_key BYTEA NOT NULL DEFAULT '\x'::bytea,
    ADD COLUMN back_encrypted_key  BYTEA NOT NULL DEFAULT '\x'::bytea;
