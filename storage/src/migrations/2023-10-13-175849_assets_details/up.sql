CREATE TABLE assets_details (
    asset_id VARCHAR(128) NOT NULL PRIMARY KEY REFERENCES assets (id) ON DELETE CASCADE,
    -- links --
    homepage VARCHAR(64),
    explorer VARCHAR(64),
    twitter VARCHAR(64),
    telegram VARCHAR(64),
    github VARCHAR(64),
    youtube VARCHAR(64),
    facebook VARCHAR(64),
    reddit VARCHAR(64),
    coingecko VARCHAR(64),
    coinmarketcap VARCHAR(64),
    discord VARCHAR(64),

    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);
