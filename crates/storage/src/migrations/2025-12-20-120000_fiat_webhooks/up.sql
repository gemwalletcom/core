CREATE TABLE fiat_webhooks (
    id SERIAL PRIMARY KEY,
    provider VARCHAR(32) NOT NULL REFERENCES fiat_providers(id),
    transaction_id VARCHAR(256),
    payload JSONB NOT NULL,
    error TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
