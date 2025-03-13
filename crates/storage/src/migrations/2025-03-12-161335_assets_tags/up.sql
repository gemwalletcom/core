CREATE TABLE tags (
    id VARCHAR(64) PRIMARY KEY,
    created_at timestamp NOT NULL default current_timestamp
);

CREATE TABLE assets_tags (
    asset_id VARCHAR(128) NOT NULL REFERENCES assets (id) ON DELETE CASCADE,
    tag_id VARCHAR(64) NOT NULL REFERENCES tags (id) ON DELETE CASCADE,
    "order" INTEGER,
    created_at timestamp NOT NULL default current_timestamp,
    PRIMARY KEY (asset_id, tag_id)
);
