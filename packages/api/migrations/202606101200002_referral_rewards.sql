-- Driver referral programme — table 3 of 3.
--
-- Immutable reward ledger: one row per reward issued to a referrer. Rows are
-- only ever inserted (never updated/deleted) so the ledger is fully auditable.
-- Insertion happens in the SAME transaction that flips the referral to
-- `rewarded` and applies the reward, guaranteeing each referral is rewarded at
-- most once.
--
-- reward_type is a Postgres ENUM to match the codebase convention (sea-orm
-- DeriveActiveEnum). reward_value is the magnitude in the unit implied by the
-- type — days for subscription_days, KES for cash_credit, unused for badge.

CREATE TYPE referral_reward_type AS ENUM (
    'subscription_days',
    'cash_credit',
    'badge'
);

CREATE TABLE referral_rewards (
    id           VARCHAR(26)           PRIMARY KEY,
    referral_id  VARCHAR(26)           NOT NULL
                 REFERENCES driver_referrals(id),
    driver_id    VARCHAR(26)           NOT NULL REFERENCES driver(id),
    reward_type  referral_reward_type  NOT NULL,
    reward_value NUMERIC(10, 2)        NOT NULL,
    issued_at    TIMESTAMPTZ           NOT NULL DEFAULT NOW(),
    -- Belt-and-braces idempotency: even if the transactional status guard were
    -- bypassed, the ledger can hold at most one reward per referral.
    UNIQUE (referral_id)
);

-- A referrer's reward history / stats aggregate by driver_id.
CREATE INDEX idx_referral_rewards_driver ON referral_rewards(driver_id);
