CREATE TABLE prices_assets (
    asset_id VARCHAR(256) PRIMARY KEY NOT NULL,
    price_id VARCHAR(256) NOT NULL REFERENCES prices (id) ON DELETE CASCADE,

    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,

    UNIQUE(asset_id)
);

SELECT diesel_manage_updated_at('prices_assets');