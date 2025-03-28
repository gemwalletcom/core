CREATE TABLE devices (
    id SERIAL PRIMARY KEY,
    device_id VARCHAR(32) NOT NULL,
    is_push_enabled boolean NOT NULL,
    platform VARCHAR(8) NOT NULL,
    platform_store VARCHAR(32),
    token VARCHAR(256) NOT NULL,
    locale VARCHAR(8) NOT NULL,
    version VARCHAR(8) NOT NULL,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,
    currency VARCHAR(8) NOT NULL REFERENCES fiat_rates (id) ON DELETE CASCADE, 
    subscriptions_version INTEGER NOT NULL DEFAULT 0,
    is_price_alerts_enabled boolean NOT NULL DEFAULT false,
    UNIQUE(device_id)
);

CREATE INDEX devices_token_idx ON devices (token);

SELECT diesel_manage_updated_at('devices');
