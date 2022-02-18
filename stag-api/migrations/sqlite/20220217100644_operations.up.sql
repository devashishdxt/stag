CREATE TABLE IF NOT EXISTS operations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    request_id TEXT,
    chain_id TEXT NOT NULL,
    address TEXT NOT NULL,
    denom TEXT NOT NULL,
    amount BLOB NOT NULL,
    operation_type TEXT NOT NULL,
    transaction_hash TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
