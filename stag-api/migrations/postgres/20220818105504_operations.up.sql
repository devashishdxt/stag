CREATE TABLE IF NOT EXISTS operations (
    id BIGSERIAL PRIMARY KEY,
    request_id TEXT,
    chain_id TEXT NOT NULL,
    port_id TEXT NOT NULL,
    operation_type JSONB NOT NULL,
    transaction_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
