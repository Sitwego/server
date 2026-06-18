-- Driver referral programme — table 1 of 3.
--
-- Every driver gets exactly ONE immutable referral code (e.g. "TRN-A3K9X2")
-- generated right after registration. Drivers share that code; a new driver
-- enters it at sign-up to create a referral relationship (see
-- driver_referrals).
--
-- IDs follow the codebase convention: ULID stored as CHAR(26), NOT uuid. The
-- driver table is `driver` (singular). Codes cascade-delete with the driver,
-- but the referral history they produced is kept (see driver_referrals).

CREATE TABLE driver_referral_codes (
    id          VARCHAR(26)  PRIMARY KEY,
    driver_id   VARCHAR(26)  NOT NULL
                REFERENCES driver(id) ON DELETE CASCADE,
    code        VARCHAR(12)  NOT NULL UNIQUE,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    -- One code per driver, immutable + never reused.
    UNIQUE (driver_id)
);

-- Lookup by code is the hot path (validation at registration).
CREATE INDEX idx_referral_codes_code ON driver_referral_codes(code);
