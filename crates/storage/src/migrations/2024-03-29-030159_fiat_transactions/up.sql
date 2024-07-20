CREATE TABLE fiat_transactions (
    id SERIAL PRIMARY KEY,

    provider_id VARCHAR(128) NOT NULL REFERENCES fiat_providers (id) ON DELETE CASCADE,
    asset_id VARCHAR(128) REFERENCES assets (id) ON DELETE CASCADE,

    symbol VARCHAR(32) NOT NULL,
    fiat_amount float NOT NULL DEFAULT 0,
    fiat_currency VARCHAR(32) NOT NULL,
    status VARCHAR(32) NOT NULL,
    provider_transaction_id VARCHAR(256) NOT NULL,
    transaction_hash VARCHAR(256),
    address VARCHAR(256),

    fee_provider float DEFAULT NULL,
    fee_network float DEFAULT NULL,
    fee_partner float DEFAULT NULL,

    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,

    UNIQUE NULLS NOT DISTINCT(provider_id, provider_transaction_id)
);

SELECT diesel_manage_updated_at('fiat_transactions');
