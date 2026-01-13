CREATE TABLE fiat_providers (
    id VARCHAR(32) PRIMARY KEY NOT NULL,
    name VARCHAR(32) NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT FALSE,
    buy_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    sell_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    priority INTEGER NULL,
    priority_threshold_bps INTEGER NULL,
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
    asset_id VARCHAR(128) REFERENCES assets (id) ON DELETE CASCADE,
    symbol VARCHAR(32) NOT NULL,
    fiat_amount float NOT NULL DEFAULT 0,
    fiat_currency VARCHAR(32) NOT NULL,
    status VARCHAR(32) NOT NULL,
    country VARCHAR(256),
    provider_transaction_id VARCHAR(256) NOT NULL,
    transaction_hash VARCHAR(256),
    address VARCHAR(256),
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,
    transaction_type VARCHAR(32) NOT NULL default 'buy',
    UNIQUE NULLS NOT DISTINCT(provider_id, provider_transaction_id)
);

SELECT diesel_manage_updated_at('fiat_transactions');

CREATE TABLE fiat_quotes (
    id VARCHAR(128) PRIMARY KEY,
    provider_id VARCHAR(128) NOT NULL REFERENCES fiat_providers (id) ON DELETE CASCADE,
    asset_id VARCHAR(128) NOT NULL REFERENCES assets (id) ON DELETE CASCADE,
    fiat_amount FLOAT NOT NULL,
    fiat_currency VARCHAR(32) NOT NULL,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

SELECT diesel_manage_updated_at('fiat_quotes');

CREATE INDEX idx_fiat_quotes_provider_id ON fiat_quotes(provider_id);
CREATE INDEX idx_fiat_quotes_asset_id ON fiat_quotes(asset_id);

CREATE TABLE fiat_quotes_requests (
    id SERIAL PRIMARY KEY,
    device_id INTEGER NOT NULL REFERENCES devices (id) ON DELETE CASCADE,
    quote_id VARCHAR(128) NOT NULL REFERENCES fiat_quotes (id) ON DELETE CASCADE,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    UNIQUE(device_id, quote_id)
);

SELECT diesel_manage_updated_at('fiat_quotes_requests');

CREATE INDEX idx_fiat_quotes_requests_device_id ON fiat_quotes_requests(device_id);
CREATE INDEX idx_fiat_quotes_requests_quote_id ON fiat_quotes_requests(quote_id);

CREATE TABLE fiat_webhooks (
    id SERIAL PRIMARY KEY,
    provider VARCHAR(32) NOT NULL REFERENCES fiat_providers(id),
    transaction_id VARCHAR(256),
    payload JSONB NOT NULL,
    error TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
