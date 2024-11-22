-- Your SQL goes here
CREATE TABLE assets_links (
    asset_id VARCHAR(128) NOT NULL PRIMARY KEY REFERENCES assets (id) ON DELETE CASCADE,
    
    name VARCHAR(128) NOT NULL,
    url VARCHAR(256) NOT NULL,

    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,

    UNIQUE(asset_id, name)
);

SELECT diesel_manage_updated_at('assets_links');
