-- Beckn BPP adapter — correlation + idempotency store.
--
-- This table is the ONLY place Beckn protocol fields live; they are deliberately
-- kept OFF the `ride` table (Inviolable Rule #6). One row per inbound Beckn
-- message: `(transaction_id, message_id)` is the idempotency anchor, so a
-- retried callback resolves to the same row instead of acting twice
-- (Inviolable Rule #4).
--
-- `ride_id` is NULL until `confirm` creates the real ride and links it back.
CREATE TABLE beckn_orders (
    -- Internal surrogate key (ULID), consistent with the rest of the schema.
    id              VARCHAR(26) NOT NULL PRIMARY KEY,

    -- Beckn correlation identifiers from the request `context`.
    transaction_id  VARCHAR     NOT NULL,
    message_id      VARCHAR     NOT NULL,

    -- The Beckn `order.id` we assign (NULL until an order exists, i.e. init/confirm).
    beckn_order_id  VARCHAR,

    -- Originating BAP + the network/registry it belongs to.
    bap_id          VARCHAR     NOT NULL,
    bap_uri         VARCHAR     NOT NULL,
    network_id      VARCHAR,

    domain          VARCHAR     NOT NULL,

    -- Protocol-side status (e.g. ACTIVE/COMPLETE/CANCELLED) and the last action
    -- processed for this message (search/select/init/confirm/...). Plain text:
    -- the protocol vocabulary evolves independently of our internal enums.
    beckn_status    VARCHAR,
    last_action     VARCHAR     NOT NULL,

    -- Link to the internal ride, populated at confirm. Nullable until then.
    ride_id         VARCHAR(26) REFERENCES ride(id) ON DELETE SET NULL,

    created_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    -- The idempotency anchor: at most one row per inbound message.
    CONSTRAINT uq_beckn_orders_txn_msg UNIQUE (transaction_id, message_id)
);

-- Lookups: by transaction (all messages in a flow) and by linked ride.
CREATE INDEX idx_beckn_orders_transaction ON beckn_orders (transaction_id);
CREATE INDEX idx_beckn_orders_ride ON beckn_orders (ride_id);
