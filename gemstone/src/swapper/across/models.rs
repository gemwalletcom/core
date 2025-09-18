use alloy_primitives::{Address, U256};
use gem_evm::across::{deployment::AcrossDeployment, fees};
use primitives::{AssetId, Chain};
use solana_primitives::types::Pubkey as SolanaPubkey;

use crate::config::swap_config::SwapReferralFee;

pub struct QuoteContext<'a> {
    pub from_amount: U256,
    pub wallet_address: Address,
    pub from_chain: Chain,
    pub to_chain: Chain,
    pub input_is_native: bool,
    pub input_asset: AssetId,
    pub output_asset: AssetId,
    pub original_output_asset: AssetId,
    pub mainnet_token: Address,
    pub capital_cost: fees::CapitalCostConfig,
    pub referral_fee: SwapReferralFee,
    pub destination_deployment: AcrossDeployment,
    pub destination_address: Option<&'a str>,
    pub output_token_decimals: u8,
}

#[derive(Clone, Debug)]
pub struct DestinationMessage {
    pub bytes: Vec<u8>,
    pub referral_fee: U256,
    pub recipient: RelayRecipient,
}

#[derive(Clone, Debug)]
pub enum RelayRecipient {
    Evm(Address),
    Solana(SolanaPubkey),
}
