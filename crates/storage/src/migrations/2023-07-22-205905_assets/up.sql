CREATE TYPE asset_type AS ENUM ('NATIVE', 'ERC20', 'BEP20', 'BEP2', 'SPL', 'SPL2022', 'TRC20', 'TOKEN', 'IBC', 'JETTON', 'SYNTH', 'ASA', 'PERPETUAL', 'SPOT');
CREATE TYPE link_type AS ENUM ('x', 'discord', 'reddit', 'telegram', 'github', 'youtube', 'facebook', 'website', 'coingecko', 'opensea', 'instagram', 'magiceden', 'coinmarketcap', 'tiktok');

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
    is_earnable boolean NOT NULL default false,
    earn_apr float,
    has_image boolean NOT NULL default false,

    UNIQUE(id)
);

SELECT diesel_manage_updated_at('assets');

CREATE TABLE tags (
    id VARCHAR(64) PRIMARY KEY,
    created_at timestamp NOT NULL default current_timestamp
);

CREATE TABLE assets_tags (
    asset_id VARCHAR(128) NOT NULL REFERENCES assets (id) ON DELETE CASCADE,
    tag_id VARCHAR(64) NOT NULL REFERENCES tags (id) ON DELETE CASCADE,
    "order" INTEGER,
    created_at timestamp NOT NULL default current_timestamp,
    PRIMARY KEY (asset_id, tag_id)
);

CREATE TABLE assets_links (
    id SERIAL PRIMARY KEY,
    asset_id VARCHAR(128) NOT NULL REFERENCES assets (id) ON DELETE CASCADE,
    link_type link_type NOT NULL,
    url VARCHAR(256) NOT NULL,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,
    UNIQUE(asset_id, link_type)
);

SELECT diesel_manage_updated_at('assets_links');

CREATE TABLE assets_addresses (
    id SERIAL PRIMARY KEY,
    chain VARCHAR(32) NOT NULL REFERENCES chains (id) ON DELETE CASCADE,
    asset_id VARCHAR(256) NOT NULL REFERENCES assets (id) ON DELETE CASCADE,
    address VARCHAR(256) NOT NULL,
    value VARCHAR(256),
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,
    UNIQUE (asset_id, address)
);

CREATE INDEX assets_addresses_chain_idx ON assets_addresses (chain);
CREATE INDEX assets_addresses_asset_id_idx ON assets_addresses (asset_id);
CREATE INDEX assets_addresses_address_idx ON assets_addresses (address);

SELECT diesel_manage_updated_at('assets_addresses');

CREATE TABLE assets_usage_ranks (
    asset_id VARCHAR(128) PRIMARY KEY REFERENCES assets (id) ON DELETE CASCADE,
    usage_rank INTEGER NOT NULL,
    updated_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('assets_usage_ranks');
