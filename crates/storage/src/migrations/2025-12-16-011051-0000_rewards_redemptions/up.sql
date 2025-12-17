CREATE TABLE rewards_redemptions_types (
    id VARCHAR(32) PRIMARY KEY
);

CREATE TABLE rewards_redemption_options (
    id VARCHAR(64) PRIMARY KEY,
    redemption_type VARCHAR(32) NOT NULL REFERENCES rewards_redemptions_types(id) ON DELETE CASCADE,
    points INT NOT NULL,
    asset_id VARCHAR(128) REFERENCES assets(id) ON DELETE CASCADE,
    value VARCHAR(64) NOT NULL,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

SELECT diesel_manage_updated_at('rewards_redemption_options');

CREATE TABLE rewards_redemptions (
    id SERIAL PRIMARY KEY,
    username VARCHAR(64) NOT NULL,
    option_id VARCHAR(64) NOT NULL REFERENCES rewards_redemption_options(id) ON DELETE CASCADE,
    transaction_id VARCHAR(512),
    status VARCHAR(32) NOT NULL,
    error VARCHAR(1024),
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

CREATE INDEX idx_rewards_redemptions_username_created_at ON rewards_redemptions(username, created_at);

SELECT diesel_manage_updated_at('rewards_redemptions');
