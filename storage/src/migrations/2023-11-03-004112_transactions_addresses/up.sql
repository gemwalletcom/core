CREATE TABLE transactions_addresses (
    id SERIAL PRIMARY KEY,
    transaction_id VARCHAR(256) NOT NULL REFERENCES transactions (id) ON DELETE CASCADE ,
    address VARCHAR(256) NOT NULL,
    UNIQUE(transaction_id, address)
);