CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TYPE webhook_kind AS ENUM ('transactions', 'support', 'support_bot', 'fiat');

CREATE TABLE webhook_endpoints (
    kind webhook_kind NOT NULL,
    sender VARCHAR(128) NOT NULL,
    secret VARCHAR(64) NOT NULL DEFAULT gen_random_uuid()::text,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,
    PRIMARY KEY (kind, sender),
    UNIQUE (secret)
);

SELECT diesel_manage_updated_at('webhook_endpoints');
