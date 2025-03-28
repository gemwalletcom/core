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
    
    link_type VARCHAR(32) NOT NULL REFERENCES link_types (id) ON DELETE CASCADE,

    url VARCHAR(256) NOT NULL,

    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,

    UNIQUE(collection_id, link_type)
);

SELECT diesel_manage_updated_at('nft_collections_links');
