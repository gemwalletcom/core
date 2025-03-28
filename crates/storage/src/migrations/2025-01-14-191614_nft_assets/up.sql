CREATE TABLE nft_types (
    id VARCHAR(32) PRIMARY KEY NOT NULL
);

CREATE TABLE nft_assets (
    id VARCHAR(512) PRIMARY KEY NOT NULL,

    collection_id VARCHAR(64) NOT NULL REFERENCES nft_collections (id) ON DELETE CASCADE,
    chain VARCHAR(64) NOT NULL REFERENCES chains (id) ON DELETE CASCADE,
    
    name VARCHAR(1024) NOT NULL,
    description VARCHAR(4096) NOT NULL,

    image_preview_url VARCHAR(512),
    image_preview_mime_type VARCHAR(64),

    image_original_url VARCHAR(512),
    image_original_mime_type VARCHAR(64),

    token_type VARCHAR(32) NOT NULL REFERENCES nft_types (id) ON DELETE CASCADE,
    token_id VARCHAR(512) NOT NULL,
    contract_address VARCHAR(512) NOT NULL,

    attributes JSONB NOT NULL,

    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('nft_assets');