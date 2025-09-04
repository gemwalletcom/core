use primitives::{Chain, ChainType, FeeUnitType};

#[derive(uniffi::Record, Debug, Clone, PartialEq)]
pub struct ChainConfig {
    pub network_id: String,
    pub transaction_timeout: u32,
    pub slip_44: i32,
    pub rank: i32,
    pub denom: Option<String>,
    pub chain_type: String,
    pub fee_unit_type: String,
    pub default_asset_type: Option<String>,
    pub account_activation_fee: Option<i32>,
    pub account_activation_fee_url: Option<String>,
    pub token_activation_fee: Option<i32>,
    pub minimum_account_balance: Option<u64>,
    pub block_time: u32,
    pub is_swap_supported: bool,
    pub is_stake_supported: bool,
    pub is_nft_supported: bool,
    pub is_memo_supported: bool,
}

pub fn get_chain_config(chain: Chain) -> ChainConfig {
    ChainConfig {
        network_id: chain.network_id().to_string(),
        transaction_timeout: chain_transaction_timeout_seconds(chain),
        slip_44: chain.as_slip44() as i32,
        rank: chain.rank(),
        denom: chain.as_denom().map(|x| x.to_string()),
        chain_type: chain.chain_type().as_ref().to_string(),
        fee_unit_type: fee_unit_type(chain).as_ref().to_string(),
        default_asset_type: chain.default_asset_type().map(|x| x.as_ref().to_string()),
        account_activation_fee: chain.account_activation_fee(),
        account_activation_fee_url: account_activation_fee_url(chain).map(|x| x.to_string()),
        token_activation_fee: chain.token_activation_fee(),
        minimum_account_balance: chain.minimum_account_balance(),
        block_time: chain.block_time(),
        is_swap_supported: chain.is_swap_supported(),
        is_stake_supported: chain.is_stake_supported(),
        is_nft_supported: chain.is_nft_supported(),
        is_memo_supported: is_memo_supported(chain),
    }
}

pub fn is_memo_supported(chain: Chain) -> bool {
    match chain.chain_type() {
        ChainType::Solana | ChainType::Cosmos | ChainType::Ton | ChainType::Xrp | ChainType::Stellar | ChainType::Algorand => true,
        ChainType::Ethereum
        | ChainType::Bitcoin
        | ChainType::Near
        | ChainType::Tron
        | ChainType::Aptos
        | ChainType::Sui
        | ChainType::Polkadot
        | ChainType::Cardano
        | ChainType::HyperCore => false,
    }
}

pub fn account_activation_fee_url(chain: Chain) -> Option<String> {
    match chain {
        Chain::Xrp => Some("https://xrpl.org/docs/concepts/accounts/reserves#base-reserve-and-owner-reserve".into()),
        Chain::Stellar => Some("https://developers.stellar.org/docs/learn/fundamentals/lumens#minimum-balance".into()),
        Chain::Algorand => Some("https://developer.algorand.org/docs/features/accounts/#minimum-balance".into()),
        _ => None,
    }
}

pub fn fee_unit_type(chain: Chain) -> FeeUnitType {
    match chain.chain_type() {
        ChainType::Bitcoin => FeeUnitType::SatVb,
        ChainType::Ethereum => FeeUnitType::Gwei,
        _ => FeeUnitType::Native,
    }
}

fn chain_transaction_timeout_seconds(chain: Chain) -> u32 {
    match chain.chain_type() {
        ChainType::Bitcoin => 1_209_600, // 2 weeks (mempool timeout)
        ChainType::Solana => chain.block_time() * 150,
        ChainType::Ethereum => chain.block_time() * 120,
        _ => chain.block_time() * 600,
    }
}
