CREATE TABLE nft_types (
    id VARCHAR(32) PRIMARY KEY NOT NULL
);

CREATE TABLE nft_assets (
    id VARCHAR(512) PRIMARY KEY NOT NULL,

    collection_id VARCHAR(64) NOT NULL REFERENCES nft_collections (id) ON DELETE CASCADE,
    chain VARCHAR(64) NOT NULL REFERENCES chains (id) ON DELETE CASCADE,
    
    name VARCHAR(256) NOT NULL,
    description VARCHAR(1024) NOT NULL,

    image_url VARCHAR(512) NOT NULL,

    token_type VARCHAR(32) NOT NULL REFERENCES nft_types (id) ON DELETE CASCADE,
    token_id VARCHAR(512) NOT NULL,

    attributes JSONB NOT NULL,

    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('nft_assets');