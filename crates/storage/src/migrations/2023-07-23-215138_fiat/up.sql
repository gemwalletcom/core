CREATE TYPE fiat_transaction_type AS ENUM ('buy', 'sell');
CREATE TYPE fiat_transaction_status AS ENUM ('complete', 'pending', 'failed', 'unknown');

CREATE TABLE fiat_providers (
    id VARCHAR(32) PRIMARY KEY NOT NULL,
    name VARCHAR(32) NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT FALSE,
    buy_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    sell_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    priority INTEGER NULL,
    priority_threshold_bps INTEGER NULL,
    payment_methods jsonb NOT NULL DEFAULT '[]'::jsonb,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('fiat_providers');

CREATE TABLE fiat_providers_countries (
    id VARCHAR(32) PRIMARY KEY NOT NULL,
    provider VARCHAR(128) NOT NULL REFERENCES fiat_providers (id) ON DELETE CASCADE,
    alpha2 VARCHAR(32) NOT NULL,
    is_allowed BOOLEAN NOT NULL,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('fiat_providers_countries');

CREATE TABLE fiat_assets (
    id VARCHAR(128) PRIMARY KEY,
    asset_id VARCHAR(128) REFERENCES assets (id) ON DELETE CASCADE,
    provider VARCHAR(128) NOT NULL REFERENCES fiat_providers (id) ON DELETE CASCADE,
    code VARCHAR(128) NOT NULL,
    symbol VARCHAR(128) NOT NULL,
    network VARCHAR(128) NULL,
    token_id VARCHAR(128) NULL,
    is_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    is_enabled_by_provider BOOLEAN NOT NULL DEFAULT TRUE,
    is_buy_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    is_sell_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    unsupported_countries jsonb,
    buy_limits jsonb,
    sell_limits jsonb,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('fiat_assets');

CREATE TABLE fiat_transactions (
    id SERIAL PRIMARY KEY,
    provider_id VARCHAR(128) NOT NULL REFERENCES fiat_providers (id) ON DELETE CASCADE,
    asset_id VARCHAR(128) NOT NULL REFERENCES assets (id) ON DELETE CASCADE,
    quote_id VARCHAR(128) NOT NULL,
    device_id INTEGER NOT NULL REFERENCES devices (id) ON DELETE CASCADE,
    wallet_id INTEGER NOT NULL REFERENCES wallets (id) ON DELETE CASCADE,
    fiat_amount float NOT NULL DEFAULT 0,
    fiat_currency VARCHAR(32) NOT NULL,
    value VARCHAR(256),
    status fiat_transaction_status NOT NULL,
    country VARCHAR(256),
    provider_transaction_id VARCHAR(256),
    transaction_hash VARCHAR(256),
    address_id INTEGER NOT NULL REFERENCES wallets_addresses (id) ON DELETE CASCADE,
    transaction_type fiat_transaction_type NOT NULL,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('fiat_transactions');

CREATE INDEX idx_fiat_transactions_provider_quote_id ON fiat_transactions(provider_id, quote_id);
CREATE UNIQUE INDEX idx_fiat_transactions_provider_quote_placeholder ON fiat_transactions(provider_id, quote_id) WHERE provider_transaction_id IS NULL;
CREATE UNIQUE INDEX idx_fiat_transactions_provider_transaction_id ON fiat_transactions(provider_id, provider_transaction_id) WHERE provider_transaction_id IS NOT NULL;
