CREATE TABLE transactions (
    id SERIAL PRIMARY KEY,
    chain VARCHAR NOT NULL,
    hash VARCHAR(256) NOT NULL,
    from_address VARCHAR(256),
    to_address VARCHAR(256),
    contract VARCHAR(256),
    memo VARCHAR(256),
    sequence INTEGER,
    block_number INTEGER NOT NULL,
    kind VARCHAR(16) NOT NULL,
    value VARCHAR(32),
    asset_id VARCHAR(64),
    fee VARCHAR(32),
    fee_asset_id VARCHAR(64),
    updated_at timestamp NOT NULL default current_timestamp,
    created_at timestamp NOT NULL default current_timestamp,
    UNIQUE(chain, hash)
);

ALTER TABLE transactions
      ADD CONSTRAINT fk_asset_id_transactions FOREIGN KEY (asset_id) 
          REFERENCES assets (id);

ALTER TABLE transactions
      ADD CONSTRAINT fk_fee_asset_id_transactions FOREIGN KEY (fee_asset_id) 
          REFERENCES assets (id);

SELECT diesel_manage_updated_at('transactions');