pub const APTOS_NATIVE_COIN: &str = "0x1::aptos_coin::AptosCoin";
pub const APTOS_TRANSFER_FUNCTION: &str = "0x1::aptos_account::transfer";
pub const ENTRY_FUNCTION_PAYLOAD_TYPE: &str = "entry_function_payload";
pub const NO_ACCOUNT_SIGNATURE_TYPE: &str = "no_account_signature";
pub const DEFAULT_MAX_GAS_AMOUNT: u64 = 1500;
pub const DEFAULT_SWAP_MAX_GAS_AMOUNT: u64 = 20000;

/// The module address for the coin info resource
pub const COIN_INFO: &str = "0x1::coin::CoinInfo";
pub const COIN_STORE: &str = "0x1::coin::CoinStore";

pub const STAKE_WITHDRAW_EVENT: &str = "0x1::coin::WithdrawEvent";
pub const STAKE_DEPOSIT_EVENT: &str = "0x1::coin::DepositEvent";

pub const FUNGIBLE_ASSET_WITHDRAW_EVENT: &str = "0x1::fungible_asset::Withdraw";
pub const FUNGIBLE_ASSET_DEPOSIT_EVENT: &str = "0x1::fungible_asset::Deposit";

pub const DELEGATION_POOL_ADD_STAKE_FUNCTION: &str = "0x1::delegation_pool::add_stake";
pub const DELEGATION_POOL_UNLOCK_FUNCTION: &str = "0x1::delegation_pool::unlock";
pub const DELEGATION_POOL_WITHDRAW_FUNCTION: &str = "0x1::delegation_pool::withdraw";

pub const DELEGATION_POOL_ADD_STAKE_EVENT: &str = "0x1::delegation_pool::AddStake";
pub const DELEGATION_POOL_UNLOCK_STAKE_EVENT: &str = "0x1::delegation_pool::UnlockStake";

pub const KNOWN_VALIDATOR_POOL: &str = "0xdb5247f859ce63dbe8940cf8773be722a60dcc594a8be9aca4b76abceb251b8e"; // Everstake
