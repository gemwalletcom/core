CREATE TABLE assets_links (
    id SERIAL PRIMARY KEY,

    asset_id VARCHAR(128) NOT NULL REFERENCES assets (id) ON DELETE CASCADE,
    
    link_type VARCHAR(32) NOT NULL REFERENCES link_types (id) ON DELETE CASCADE,

    url VARCHAR(256) NOT NULL,

    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,

    UNIQUE(asset_id, link_type)
);

SELECT diesel_manage_updated_at('assets_links');
