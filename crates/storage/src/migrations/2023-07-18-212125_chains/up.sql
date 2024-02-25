CREATE TABLE chains (
    id VARCHAR(32) PRIMARY KEY NOT NULL,

    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp
);

SELECT diesel_manage_updated_at('chains');
