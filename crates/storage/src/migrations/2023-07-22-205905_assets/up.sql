CREATE TABLE assets (
    id VARCHAR(128) PRIMARY KEY,
    chain VARCHAR(32) NOT NULL REFERENCES chains (id) ON DELETE CASCADE,
    token_id VARCHAR(128),
    asset_type asset_type NOT NULL,
    name VARCHAR(64) NOT NULL,
    symbol VARCHAR(16) NOT NULL,
    decimals INTEGER NOT NULL,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,
    rank INTEGER NOT NULL DEFAULT 0,

    is_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    is_buyable boolean NOT NULL default false,
    is_sellable boolean NOT NULL default false,
    is_swappable boolean NOT NULL default false,
    is_stakeable boolean NOT NULL default false,
    staking_apr float,

    UNIQUE(id)
);

SELECT diesel_manage_updated_at('assets');
