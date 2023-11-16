CREATE TABLE swap_assets (
    id SERIAL PRIMARY KEY,
    asset_id VARCHAR(128) NOT NULL REFERENCES assets (id) ON DELETE CASCADE,
    UNIQUE(asset_id),

    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('swap_assets');