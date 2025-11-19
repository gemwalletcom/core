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

    device_id VARCHAR(128) NOT NULL REFERENCES devices (device_id) ON DELETE CASCADE,
    quote_id VARCHAR(128) NOT NULL REFERENCES fiat_quotes (id) ON DELETE CASCADE,

    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,

    UNIQUE(device_id, quote_id)
);

SELECT diesel_manage_updated_at('fiat_quotes_requests');

CREATE INDEX idx_fiat_quotes_requests_device_id ON fiat_quotes_requests(device_id);
CREATE INDEX idx_fiat_quotes_requests_quote_id ON fiat_quotes_requests(quote_id);
