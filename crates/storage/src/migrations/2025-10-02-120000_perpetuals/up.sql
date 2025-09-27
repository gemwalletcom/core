CREATE TABLE perpetuals (
    id VARCHAR(128) PRIMARY KEY,
    name VARCHAR(128) NOT NULL,
    provider VARCHAR(32) NOT NULL,
    asset_id VARCHAR(256) NOT NULL REFERENCES assets (id) ON DELETE CASCADE,
    identifier VARCHAR(128) NOT NULL,
    price DOUBLE PRECISION NOT NULL,
    price_percent_change_24h DOUBLE PRECISION NOT NULL,
    open_interest DOUBLE PRECISION NOT NULL,
    volume_24h DOUBLE PRECISION NOT NULL,
    funding DOUBLE PRECISION NOT NULL,
    leverage INTEGER[] NOT NULL,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE perpetuals_assets
(
    id             SERIAL PRIMARY KEY,
    perpetual_id   VARCHAR(128) NOT NULL REFERENCES perpetuals (id) ON DELETE CASCADE,
    asset_id       VARCHAR(256) NOT NULL REFERENCES assets (id) ON DELETE CASCADE,
    updated_at     TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at     TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

SELECT diesel_manage_updated_at('perpetuals');
SELECT diesel_manage_updated_at('perpetuals_assets');
