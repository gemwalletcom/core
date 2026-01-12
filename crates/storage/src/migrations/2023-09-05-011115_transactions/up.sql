CREATE TYPE transaction_type AS ENUM ('transfer', 'transferNFT', 'swap', 'tokenApproval', 'stakeDelegate', 'stakeUndelegate', 'stakeRewards', 'stakeRedelegate', 'stakeWithdraw', 'stakeFreeze', 'stakeUnfreeze', 'assetActivation', 'smartContractCall', 'perpetualOpenPosition', 'perpetualClosePosition', 'perpetualModifyPosition');

CREATE TABLE transactions
(
    id           BIGSERIAL PRIMARY KEY,
    chain        VARCHAR(16)  NOT NULL REFERENCES chains (id) ON DELETE CASCADE,
    hash         VARCHAR(128) NOT NULL,
    from_address VARCHAR(256),
    to_address   VARCHAR(256),
    memo         VARCHAR(256),
    state        VARCHAR(16)  NOT NULL,
    kind         transaction_type NOT NULL,
    value        VARCHAR(256),
    asset_id     VARCHAR      NOT NULL REFERENCES assets (id) ON DELETE CASCADE,
    fee          VARCHAR(32),
    utxo_inputs  jsonb,
    utxo_outputs jsonb,
    fee_asset_id VARCHAR      NOT NULL REFERENCES assets (id) ON DELETE CASCADE,
    metadata     jsonb,
    created_at   timestamp    NOT NULL default current_timestamp,
    updated_at   timestamp    NOT NULL default current_timestamp,
    CONSTRAINT transactions_chain_hash_unique UNIQUE (chain, hash)
);

SELECT diesel_manage_updated_at('transactions');

CREATE INDEX transactions_created_at_idx ON transactions (created_at DESC);
CREATE INDEX transactions_asset_id_idx ON transactions (asset_id);
