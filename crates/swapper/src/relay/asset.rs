use gem_evm::address::ethereum_address_checksum;
use gem_solana::{SYSTEM_PROGRAM_ID, WSOL_TOKEN_ADDRESS};
use primitives::{AssetId, Chain, ChainType};

use crate::asset::EVM_ZERO_ADDRESS;

fn is_native_currency(chain: Chain, currency: &str) -> bool {
    match chain {
        Chain::Bitcoin => true,
        Chain::Solana => currency == SYSTEM_PROGRAM_ID || currency == WSOL_TOKEN_ADDRESS,
        _ if currency == EVM_ZERO_ADDRESS => true,
        _ => false,
    }
}

pub fn map_currency_to_asset_id(chain: Chain, currency: &str) -> AssetId {
    if is_native_currency(chain, currency) {
        return AssetId::from_chain(chain);
    }
    if let ChainType::Ethereum = chain.chain_type()
        && let Ok(address) = ethereum_address_checksum(currency)
    {
        return AssetId::from_token(chain, &address);
    }
    AssetId::from_token(chain, currency)
}
