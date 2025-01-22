CREATE TABLE nft_links (
    id SERIAL PRIMARY KEY,

    collection_id VARCHAR(128) NOT NULL REFERENCES nft_collections (id) ON DELETE CASCADE,
    
    link_type VARCHAR(32) NOT NULL REFERENCES link_types (id) ON DELETE CASCADE,

    url VARCHAR(256) NOT NULL,

    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,

    UNIQUE(collection_id, link_type)
);

SELECT diesel_manage_updated_at('nft_links');
