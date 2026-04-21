CREATE TABLE prices_providers (
    id VARCHAR(32) PRIMARY KEY NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT FALSE,
    priority INTEGER NOT NULL DEFAULT 0,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('prices_providers');

CREATE TABLE prices (
    id VARCHAR(256) PRIMARY KEY NOT NULL,
    provider VARCHAR(32) NOT NULL REFERENCES prices_providers (id) ON DELETE CASCADE,
    provider_price_id VARCHAR(256) NOT NULL,
    price float NOT NULL DEFAULT 0,
    price_change_percentage_24h float NOT NULL DEFAULT 0,
    market_cap float NOT NULL DEFAULT 0,
    market_cap_fdv float NOT NULL DEFAULT 0,
    market_cap_rank INTEGER,
    total_volume float NOT NULL DEFAULT 0,
    circulating_supply float NOT NULL DEFAULT 0,
    total_supply float NOT NULL DEFAULT 0,
    max_supply float NOT NULL DEFAULT 0,
    last_updated_at timestamp NOT NULL default current_timestamp,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,
    all_time_high_date timestamp,
    all_time_low_date  timestamp,
    all_time_high      float NOT NULL DEFAULT 0,
    all_time_low       float NOT NULL DEFAULT 0,
    UNIQUE (provider, provider_price_id)
);

SELECT diesel_manage_updated_at('prices');

CREATE INDEX prices_provider_idx ON prices (provider);
CREATE INDEX prices_market_cap_idx ON prices (market_cap DESC);
CREATE INDEX prices_last_updated_at_idx ON prices (last_updated_at);

CREATE TABLE prices_assets (
    asset_id VARCHAR(256) NOT NULL REFERENCES assets (id) ON DELETE CASCADE,
    price_id VARCHAR(256) NOT NULL REFERENCES prices (id) ON DELETE CASCADE,
    provider VARCHAR(32) NOT NULL REFERENCES prices_providers (id) ON DELETE CASCADE,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,
    PRIMARY KEY (asset_id, provider)
);

SELECT diesel_manage_updated_at('prices_assets');

CREATE INDEX prices_assets_price_id_idx ON prices_assets (price_id);
CREATE INDEX prices_assets_provider_idx ON prices_assets (provider);
