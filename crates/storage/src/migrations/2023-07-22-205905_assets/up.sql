CREATE TABLE assets (
    id VARCHAR(128) PRIMARY KEY,
    chain VARCHAR(32) NOT NULL REFERENCES chains (id) ON DELETE CASCADE,
    token_id VARCHAR(128),
    asset_type VARCHAR(16) NOT NULL REFERENCES assets_types (id) ON DELETE CASCADE,
    name VARCHAR(64) NOT NULL,
    symbol VARCHAR(16) NOT NULL,
    decimals INTEGER NOT NULL,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    rank INTEGER NOT NULL DEFAULT 0,
    UNIQUE(id)
);

SELECT diesel_manage_updated_at('assets');
