CREATE TABLE transactions_assets (
    id SERIAL PRIMARY KEY,
    chain_id VARCHAR(32) NOT NULL REFERENCES chains (id) ON DELETE CASCADE,
    transaction_id VARCHAR(256) NOT NULL REFERENCES transactions (id) ON DELETE CASCADE ,
    asset_id VARCHAR(256) NOT NULL REFERENCES assets (id) ON DELETE CASCADE,
    created_at timestamp NOT NULL default current_timestamp,
    UNIQUE(transaction_id, asset_id)
);