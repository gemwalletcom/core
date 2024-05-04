CREATE TABLE transactions (
    id VARCHAR(256) NOT NULL PRIMARY KEY,
    chain VARCHAR(16) NOT NULL REFERENCES chains (id) ON DELETE CASCADE,
    hash VARCHAR(256) NOT NULL,
    from_address VARCHAR(256),
    to_address VARCHAR(256),
    contract VARCHAR(256),
    memo VARCHAR(256),
    sequence INTEGER,
    block_number INTEGER NOT NULL,
    state VARCHAR(16) NOT NULL,
    kind VARCHAR(16) NOT NULL,
    value VARCHAR(256),
    asset_id VARCHAR NOT NULL REFERENCES assets (id) ON DELETE CASCADE,
    fee VARCHAR(32),
    utxo_inputs jsonb,
    utxo_outputs jsonb,
    metadata jsonb,
    fee_asset_id VARCHAR NOT NULL REFERENCES assets (id) ON DELETE CASCADE,
    block_created_at timestamp NOT NULL default current_timestamp,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,
    UNIQUE(chain, hash)
);

SELECT diesel_manage_updated_at('transactions');

CREATE INDEX transactions_created_at_idx ON transactions (created_at DESC);