-- Driver referral programme — table 2 of 3.
--
-- One row per referral relationship: who referred whom, the code used, and the
-- lifecycle status. A referral is created `pending` at registration, moves to
-- `completed` when the referred driver is activated (KYC approved / go-live),
-- and to `rewarded` once the referrer's reward has been issued.
--
-- The status set is modelled as a Postgres ENUM (not a CHECK) to match the
-- codebase convention (cf. document_review_status), so sea-orm can map it with
-- DeriveActiveEnum.

CREATE TYPE referral_status AS ENUM (
    'pending',
    'completed',
    'rewarded',
    'expired'
);

CREATE TABLE driver_referrals (
    id            VARCHAR(26)      PRIMARY KEY,
    -- Referrer + referred are kept even if either driver later deletes their
    -- account, so reward history stays auditable — hence NO cascade here.
    referrer_id   VARCHAR(26)      NOT NULL REFERENCES driver(id),
    referred_id   VARCHAR(26)      NOT NULL REFERENCES driver(id),
    code_used     VARCHAR(12)      NOT NULL,
    status        referral_status  NOT NULL DEFAULT 'pending',
    referred_at   TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    completed_at  TIMESTAMPTZ,
    rewarded_at   TIMESTAMPTZ,
    -- A driver can be referred at most once (fraud guard: no duplicate
    -- referrals for the same new driver).
    UNIQUE (referred_id)
);

-- A referrer's dashboard lists all the drivers they referred.
CREATE INDEX idx_driver_referrals_referrer ON driver_referrals(referrer_id);

-- The reward trigger (driver activation) looks up a pending referral by the
-- referred driver; the UNIQUE(referred_id) constraint already indexes that.
