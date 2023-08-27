CREATE TABLE prices (
    asset_id VARCHAR PRIMARY KEY NOT NULL,
    coin_id VARCHAR NOT NULL,
    price float NOT NULL DEFAULT 0,
    price_change_percentage_24h float NOT NULL DEFAULT 0,
    market_cap float NOT NULL DEFAULT 0,
    market_cap_rank INTEGER NOT NULL DEFAULT 0,
    total_volume float NOT NULL DEFAULT 0,
    last_updated_at timestamp NOT NULL,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,
    UNIQUE(asset_id)
);