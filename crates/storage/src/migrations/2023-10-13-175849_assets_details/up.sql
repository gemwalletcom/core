CREATE TABLE assets_details (
    asset_id VARCHAR(128) NOT NULL PRIMARY KEY REFERENCES assets (id) ON DELETE CASCADE,
    -- links --
    homepage VARCHAR(128),
    explorer VARCHAR(128),
    twitter VARCHAR(128),
    telegram VARCHAR(128),
    github VARCHAR(128),
    youtube VARCHAR(128),
    facebook VARCHAR(128),
    reddit VARCHAR(128),
    coingecko VARCHAR(128),
    coinmarketcap VARCHAR(128),
    discord VARCHAR(128),
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,
    is_buyable boolean NOT NULL default false,
    is_sellable boolean NOT NULL default false,
    is_swappable boolean NOT NULL default false,
    is_stakeable boolean NOT NULL default false,

    staking_apr float
);

SELECT diesel_manage_updated_at('assets_details');
