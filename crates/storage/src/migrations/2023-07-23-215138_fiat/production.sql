-- Manual production follow-up for existing databases after folding the fiat schema
-- changes into the original Diesel migration.
-- Rebuild fiat_transactions instead of mutating it in place.
-- Assumes production still has the older fiat tables, including fiat_quotes_requests.

BEGIN;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'fiat_transaction_type') THEN
        CREATE TYPE fiat_transaction_type AS ENUM ('buy', 'sell');
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'fiat_transaction_status') THEN
        CREATE TYPE fiat_transaction_status AS ENUM ('complete', 'pending', 'failed', 'unknown');
    END IF;
END
$$;

ALTER TABLE IF EXISTS fiat_quotes
ADD COLUMN IF NOT EXISTS value VARCHAR(256);

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM information_schema.tables
        WHERE table_schema = 'public' AND table_name = 'fiat_transactions'
    ) THEN
        RAISE EXCEPTION 'production.sql expects legacy fiat_transactions table to exist';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM information_schema.tables
        WHERE table_schema = 'public' AND table_name = 'fiat_quotes'
    ) THEN
        RAISE EXCEPTION 'production.sql expects legacy fiat_quotes table to exist';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM information_schema.tables
        WHERE table_schema = 'public' AND table_name = 'fiat_quotes_requests'
    ) THEN
        RAISE EXCEPTION 'production.sql expects legacy fiat_quotes_requests table to exist';
    END IF;

    IF EXISTS (SELECT 1 FROM fiat_transactions WHERE asset_id IS NULL) THEN
        RAISE EXCEPTION 'production.sql cannot migrate fiat_transactions rows with NULL asset_id';
    END IF;

    IF EXISTS (
        SELECT 1
        FROM fiat_transactions AS old
        LEFT JOIN fiat_quotes_requests AS quote_requests
            ON old.provider_transaction_id = quote_requests.quote_id
        WHERE COALESCE(quote_requests.quote_id, old.provider_transaction_id) IS NULL
    ) THEN
        RAISE EXCEPTION 'production.sql cannot derive quote_id for one or more fiat_transactions rows';
    END IF;

    IF EXISTS (
        SELECT 1
        FROM (
            SELECT COALESCE(quote_requests.quote_id, old.provider_transaction_id) AS quote_id
            FROM fiat_transactions AS old
            LEFT JOIN fiat_quotes_requests AS quote_requests
                ON old.provider_transaction_id = quote_requests.quote_id
        ) AS mapped_quotes
        GROUP BY quote_id
        HAVING COUNT(*) > 1
    ) THEN
        RAISE EXCEPTION 'production.sql would create duplicate fiat_transactions.quote_id values';
    END IF;
END
$$;

ALTER TABLE fiat_transactions RENAME TO fiat_transactions_old;

ALTER SEQUENCE IF EXISTS fiat_transactions_id_seq RENAME TO fiat_transactions_old_id_seq;

CREATE TABLE fiat_transactions (
    id SERIAL PRIMARY KEY,
    provider_id VARCHAR(128) NOT NULL REFERENCES fiat_providers (id) ON DELETE CASCADE,
    asset_id VARCHAR(128) NOT NULL REFERENCES assets (id) ON DELETE CASCADE,
    quote_id VARCHAR(128) NOT NULL,
    device_id INTEGER REFERENCES devices (id) ON DELETE CASCADE,
    fiat_amount FLOAT NOT NULL DEFAULT 0,
    fiat_currency VARCHAR(32) NOT NULL,
    value VARCHAR(256),
    status fiat_transaction_status NOT NULL,
    country VARCHAR(256),
    provider_transaction_id VARCHAR(256),
    transaction_hash VARCHAR(256),
    address VARCHAR(256),
    transaction_type fiat_transaction_type NOT NULL,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

SELECT diesel_manage_updated_at('fiat_transactions');

INSERT INTO fiat_transactions (
    id,
    provider_id,
    asset_id,
    quote_id,
    device_id,
    fiat_amount,
    fiat_currency,
    value,
    status,
    country,
    provider_transaction_id,
    transaction_hash,
    address,
    transaction_type,
    updated_at,
    created_at
)
SELECT
    old.id,
    old.provider_id,
    old.asset_id,
    COALESCE(quote_requests.quote_id, old.provider_transaction_id),
    quote_requests.device_id,
    old.fiat_amount,
    old.fiat_currency,
    quotes.value,
    old.status::fiat_transaction_status,
    old.country,
    old.provider_transaction_id,
    old.transaction_hash,
    old.address,
    old.transaction_type::fiat_transaction_type,
    old.updated_at,
    old.created_at
FROM fiat_transactions_old AS old
LEFT JOIN fiat_quotes_requests AS quote_requests
    ON old.provider_transaction_id = quote_requests.quote_id
LEFT JOIN fiat_quotes AS quotes
    ON COALESCE(quote_requests.quote_id, old.provider_transaction_id) = quotes.id;

SELECT setval(
    'fiat_transactions_id_seq',
    COALESCE((SELECT MAX(id) FROM fiat_transactions), 1),
    EXISTS (SELECT 1 FROM fiat_transactions)
);

DROP TABLE fiat_transactions_old;
DROP TABLE IF EXISTS fiat_webhooks;
DROP TABLE IF EXISTS fiat_quotes_requests;
DROP TABLE IF EXISTS fiat_quotes;

CREATE UNIQUE INDEX idx_fiat_transactions_quote_id ON fiat_transactions (quote_id);
CREATE UNIQUE INDEX idx_fiat_transactions_provider_transaction_id ON fiat_transactions (provider_id, provider_transaction_id) WHERE provider_transaction_id IS NOT NULL;

COMMIT;
