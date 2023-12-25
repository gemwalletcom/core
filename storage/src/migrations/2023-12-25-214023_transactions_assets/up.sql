CREATE TABLE transactions_assets (
    id SERIAL PRIMARY KEY,
    transaction_id VARCHAR(256) NOT NULL REFERENCES transactions (id) ON DELETE CASCADE ,
    asset_id VARCHAR(256) NOT NULL REFERENCES assets (id) ON DELETE CASCADE,
    UNIQUE(transaction_id, asset_id)
);