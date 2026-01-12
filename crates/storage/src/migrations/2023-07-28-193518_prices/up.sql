CREATE TABLE prices (
    id VARCHAR(256) PRIMARY KEY NOT NULL,
    price float NOT NULL DEFAULT 0,
    price_change_percentage_24h float NOT NULL DEFAULT 0,
    market_cap float NOT NULL DEFAULT 0,
    market_cap_fdv float NOT NULL DEFAULT 0,
    market_cap_rank INTEGER NOT NULL DEFAULT 0,
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
    all_time_low       float NOT NULL DEFAULT 0
);

SELECT diesel_manage_updated_at('prices');

CREATE INDEX prices_market_cap_idx ON prices (market_cap DESC);

CREATE TABLE prices_assets (
    asset_id VARCHAR(256) PRIMARY KEY NOT NULL REFERENCES assets (id) ON DELETE CASCADE,
    price_id VARCHAR(256) NOT NULL REFERENCES prices (id) ON DELETE CASCADE,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,
    UNIQUE(asset_id)
);

SELECT diesel_manage_updated_at('prices_assets');

CREATE INDEX prices_assets_price_id_idx ON prices_assets (price_id);

CREATE TABLE prices_dex_providers (
    id VARCHAR(32) PRIMARY KEY NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT FALSE,
    priority INTEGER NOT NULL DEFAULT 0,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('prices_dex_providers');

CREATE TABLE prices_dex (
    id VARCHAR(256) PRIMARY KEY NOT NULL,
    provider VARCHAR(32) NOT NULL REFERENCES prices_dex_providers (id) ON DELETE CASCADE,
    price float NOT NULL DEFAULT 0,
    last_updated_at timestamp NOT NULL default current_timestamp,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('prices_dex');

CREATE TABLE prices_dex_assets (
    asset_id VARCHAR(256) NOT NULL REFERENCES assets (id) ON DELETE CASCADE,
    price_feed_id VARCHAR(256) NOT NULL REFERENCES prices_dex (id) ON DELETE CASCADE,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,
    PRIMARY KEY (asset_id, price_feed_id)
);

SELECT diesel_manage_updated_at('prices_dex_assets');
