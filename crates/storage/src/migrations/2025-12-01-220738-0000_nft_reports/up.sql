CREATE TABLE nft_reports (
    id SERIAL PRIMARY KEY,

    asset_id VARCHAR(512) REFERENCES nft_assets (id) ON DELETE CASCADE,
    collection_id VARCHAR(512) NOT NULL REFERENCES nft_collections (id) ON DELETE CASCADE,

    device_id VARCHAR(64) NOT NULL REFERENCES devices (id) ON DELETE CASCADE,
    reason VARCHAR(1024),
    reviewed BOOLEAN NOT NULL DEFAULT false,

    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('nft_reports');
