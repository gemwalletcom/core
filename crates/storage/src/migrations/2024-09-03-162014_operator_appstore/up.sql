CREATE TABLE IF NOT EXISTS operator_appstore_positions (
    id SERIAL PRIMARY KEY,
    store           VARCHAR(32) NOT NULL,
    app             VARCHAR(128) NOT NULL,
    keyword         VARCHAR(128) NOT NULL,
    country         VARCHAR(64) NOT NULL,
    position        INTEGER NOT NULL,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

CREATE UNIQUE INDEX uniq_store_app_keyword_country_date
ON operator_appstore_positions (store, app, keyword, country, (created_at::date));

CREATE TABLE IF NOT EXISTS operator_appstore_information (
    id SERIAL PRIMARY KEY,
    store                           VARCHAR(32) NOT NULL,
    app                             VARCHAR(128) NOT NULL,
    country                         VARCHAR(64) NOT NULL,
    title                           VARCHAR(128) NOT NULL,
    version                         VARCHAR(64) NOT NULL,
    ratings                         FLOAT,
    average_rating                  FLOAT,
    release_date                    TIMESTAMP NOT NULL,
    current_version_release_date    TIMESTAMP NOT NULL,
    updated_at                      TIMESTAMP NOT NULL DEFAULT current_timestamp,
    created_at                      TIMESTAMP NOT NULL DEFAULT current_timestamp
);

CREATE UNIQUE INDEX uniq_store_app_country_date
ON operator_appstore_information (store, app, country, (created_at::date));

SELECT diesel_manage_updated_at('operator_appstore_positions');
SELECT diesel_manage_updated_at('operator_appstore_information');
