CREATE TABLE fiat_providers (
    id VARCHAR(32) PRIMARY KEY NOT NULL,
    name VARCHAR(32) NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT FALSE,
    buy_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    sell_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    priority INTEGER NULL,
    priority_threshold_bps INTEGER NULL,

    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('fiat_providers');


CREATE TABLE fiat_providers_countries (
    id VARCHAR(32) PRIMARY KEY NOT NULL,
    provider VARCHAR(128) NOT NULL REFERENCES fiat_providers (id) ON DELETE CASCADE,
    alpha2 VARCHAR(32) NOT NULL, 
    is_allowed BOOLEAN NOT NULL,

    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('fiat_providers_countries');