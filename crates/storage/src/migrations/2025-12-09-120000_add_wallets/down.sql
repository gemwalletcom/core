DROP INDEX IF EXISTS wallets_subscriptions_device_chain_idx;
DROP INDEX IF EXISTS wallets_subscriptions_chain_wallet_address_id_idx;
DROP INDEX IF EXISTS wallets_subscriptions_wallet_address_id_idx;
DROP INDEX IF EXISTS wallets_subscriptions_device_id_idx;
DROP INDEX IF EXISTS wallets_subscriptions_wallet_id_idx;
DROP TABLE IF EXISTS wallets_subscriptions;

DROP TABLE IF EXISTS wallets_addresses;

DROP TABLE IF EXISTS wallets;
DROP TYPE IF EXISTS wallet_source;
DROP TYPE IF EXISTS wallet_type;
