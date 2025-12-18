CREATE TABLE config (
    key VARCHAR(64) PRIMARY KEY,
    value VARCHAR(256) NOT NULL,
    default_value VARCHAR(256) NOT NULL,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('config');
