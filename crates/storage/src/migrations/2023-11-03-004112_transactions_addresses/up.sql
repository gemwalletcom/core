CREATE TABLE transactions_addresses
(
    id             SERIAL PRIMARY KEY,
    transaction_id VARCHAR(256) NOT NULL REFERENCES transactions (id) ON DELETE CASCADE,
    asset_id       VARCHAR(256) NOT NULL REFERENCES assets (id) ON DELETE CASCADE,
    address        VARCHAR(256) NOT NULL,
    UNIQUE (transaction_id, address, asset_id)
);

CREATE INDEX transactions_addresses_asset_id_idx ON transactions_addresses (asset_id);
CREATE INDEX transactions_addresses_address_idx ON transactions_addresses (address);
CREATE INDEX transactions_addresses_transaction_id_idx ON transactions_addresses (transaction_id);