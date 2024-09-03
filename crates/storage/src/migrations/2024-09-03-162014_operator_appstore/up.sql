CREATE TABLE IF NOT EXISTS operator_appstore_positions (
    id SERIAL PRIMARY KEY,
    store           VARCHAR(32) NOT NULL,
    app             VARCHAR(128) NOT NULL,
    keyword         VARCHAR(128) NOT NULL,
    country         VARCHAR(64) NOT NULL,
    position        INTEGER NOT NULL,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,

    UNIQUE NULLS NOT DISTINCT(store, app, keyword, country)
);

CREATE TABLE IF NOT EXISTS operator_appstore_information (
    id SERIAL PRIMARY KEY,
    store           VARCHAR(32) NOT NULL,
    app             VARCHAR(128) NOT NULL,
    country         VARCHAR(64) NOT NULL,
    ratings         FLOAT,
    average_rating  FLOAT,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,

    UNIQUE NULLS NOT DISTINCT(store, app, country)
);

SELECT diesel_manage_updated_at('operator_appstore_positions');
SELECT diesel_manage_updated_at('operator_appstore_information');
