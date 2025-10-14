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
