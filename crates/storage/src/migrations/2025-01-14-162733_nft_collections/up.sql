CREATE TYPE link_type AS ENUM ('x', 'discord', 'reddit', 'telegram', 'github', 'youtube', 'facebook', 'website', 'coingecko', 'opensea', 'instagram', 'magiceden', 'coinmarketcap', 'tiktok');
CREATE TYPE nft_type AS ENUM ('ERC721', 'ERC1155', 'SPL', 'JETTON');

CREATE TABLE nft_collections (
    id VARCHAR(512) PRIMARY KEY NOT NULL,

    chain VARCHAR(64) NOT NULL REFERENCES chains (id) ON DELETE CASCADE,

    name VARCHAR(1024) NOT NULL,
    description VARCHAR(4096) NOT NULL,
    symbol VARCHAR(128),

    owner VARCHAR(128),
    contract_address VARCHAR(128) NOT NULL,

    image_preview_url VARCHAR(512),
    image_preview_mime_type VARCHAR(64),

    is_verified BOOLEAN NOT NULL default false,
    is_enabled BOOLEAN NOT NULL default true,

    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('nft_collections');

CREATE TABLE nft_collections_links (
    id SERIAL PRIMARY KEY,

    collection_id VARCHAR(128) NOT NULL REFERENCES nft_collections (id) ON DELETE CASCADE,

    link_type link_type NOT NULL,

    url VARCHAR(256) NOT NULL,

    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,

    UNIQUE(collection_id, link_type)
);

SELECT diesel_manage_updated_at('nft_collections_links');

CREATE TABLE nft_assets (
    id VARCHAR(512) PRIMARY KEY NOT NULL,

    collection_id VARCHAR(64) NOT NULL REFERENCES nft_collections (id) ON DELETE CASCADE,
    chain VARCHAR(64) NOT NULL REFERENCES chains (id) ON DELETE CASCADE,

    name VARCHAR(1024) NOT NULL,
    description VARCHAR(4096) NOT NULL,

    image_preview_url VARCHAR(512),
    image_preview_mime_type VARCHAR(64),

    resource_url VARCHAR(512),
    resource_mime_type VARCHAR(64),

    token_type nft_type NOT NULL,
    token_id VARCHAR(512) NOT NULL,
    contract_address VARCHAR(512) NOT NULL,

    attributes JSONB NOT NULL,

    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('nft_assets');

CREATE TABLE nft_reports (
    id SERIAL PRIMARY KEY,

    asset_id VARCHAR(512) REFERENCES nft_assets (id) ON DELETE CASCADE,
    collection_id VARCHAR(512) NOT NULL REFERENCES nft_collections (id) ON DELETE CASCADE,

    device_id INTEGER NOT NULL REFERENCES devices (id) ON DELETE CASCADE,
    reason VARCHAR(1024),
    reviewed BOOLEAN NOT NULL DEFAULT false,

    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('nft_reports');
