CREATE TABLE IF NOT EXISTS operator_appstore_reviews (
    id SERIAL PRIMARY KEY,
    store           VARCHAR(32) NOT NULL,
    app             VARCHAR(128) NOT NULL,
    country         VARCHAR(64) NOT NULL,
    review_id       VARCHAR(64) NOT NULL,
    title           VARCHAR(256) NOT NULL,
    content         VARCHAR(4096) NOT NULL,
    author          VARCHAR(128) NOT NULL,
    version         VARCHAR(64) NOT NULL,
    rating          INTEGER NOT NULL,

    updated_at      TIMESTAMP NOT NULL DEFAULT current_timestamp,
    created_at      TIMESTAMP NOT NULL DEFAULT current_timestamp
);

CREATE UNIQUE INDEX uniq_store_app_country_review_id_date
ON operator_appstore_reviews (store, app, country, review_id, (created_at::date));

SELECT diesel_manage_updated_at('operator_appstore_reviews');
