-- Driver cash wallet — backs the `cash_credit` referral reward (the primary
-- reward type) and any future cash crediting.
--
-- Two tables:
--   * driver_wallets       — current balance, one row per driver.
--   * wallet_transactions  — immutable, append-only ledger. Every balance change
--     writes one signed row carrying the resulting `balance_after`, so the
--     balance is always reconstructable and fully auditable.
--
-- Crediting happens inside the SAME transaction that issues a referral reward,
-- so a reward and its wallet credit commit together or not at all.

CREATE TABLE driver_wallets (
    id          VARCHAR(26)     PRIMARY KEY,
    driver_id   VARCHAR(26)     NOT NULL UNIQUE
                REFERENCES driver(id) ON DELETE CASCADE,
    balance     NUMERIC(12, 2)  NOT NULL DEFAULT 0,
    currency    VARCHAR(3)      NOT NULL DEFAULT 'KES',
    created_at  TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE TABLE wallet_transactions (
    id            VARCHAR(26)     PRIMARY KEY,
    wallet_id     VARCHAR(26)     NOT NULL REFERENCES driver_wallets(id),
    -- Denormalised so a driver's full ledger can be read without a join.
    driver_id     VARCHAR(26)     NOT NULL,
    -- Signed: positive = credit (money in), negative = debit (money out).
    amount        NUMERIC(12, 2)  NOT NULL,
    -- Wallet balance immediately AFTER applying this row.
    balance_after NUMERIC(12, 2)  NOT NULL,
    -- What caused the movement, e.g. 'referral_reward'.
    reference     VARCHAR(40)     NOT NULL,
    -- Optional pointer to the source row (e.g. the driver_referrals id), so a
    -- given source can credit the wallet at most once where it matters.
    reference_id  VARCHAR(26),
    created_at    TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

-- A driver's ledger, newest first.
CREATE INDEX idx_wallet_transactions_driver
    ON wallet_transactions(driver_id, created_at DESC);

-- Idempotency for sourced credits: a (reference, reference_id) pair can appear
-- at most once (e.g. one referral can credit the wallet only once).
CREATE UNIQUE INDEX uq_wallet_transactions_reference
    ON wallet_transactions(reference, reference_id)
    WHERE reference_id IS NOT NULL;
