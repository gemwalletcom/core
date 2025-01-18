CREATE TABLE assets_links (
    id SERIAL PRIMARY KEY,

    asset_id VARCHAR(128) NOT NULL REFERENCES assets (id) ON DELETE CASCADE,
    
    name VARCHAR(32) NOT NULL REFERENCES link_types (id) ON DELETE CASCADE,

    url VARCHAR(256) NOT NULL,

    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,

    UNIQUE(asset_id, name)
);

SELECT diesel_manage_updated_at('assets_links');
