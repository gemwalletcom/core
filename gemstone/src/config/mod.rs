pub mod bitcoin_chain;
pub mod docs;
pub mod evm_chain;
pub mod node;
pub mod public;
pub mod social;
pub mod stake;
pub mod swap_config;
pub mod validators;
pub mod wallet_connect;

use crate::chain::ChainConfig;
use gem_solana;
use primitives::{
    node_config::{self, Node},
    BitcoinChain, Chain, EVMChain, SolanaTokenProgramId, StakeChain,
};
use std::{collections::HashMap, str::FromStr};
use {
    bitcoin_chain::{get_bitcoin_chain_config, BitcoinChainConfig},
    docs::{get_docs_url, DocsUrl},
    evm_chain::{get_evm_chain_config, EVMChainConfig},
    public::{get_public_url, PublicUrl, ASSETS_URL},
    social::{get_social_url, get_social_url_deeplink, SocialUrl},
    stake::{get_stake_config, StakeChainConfig},
    swap_config::{get_swap_config, SwapConfig},
    validators::get_validators,
    wallet_connect::{get_wallet_connect_config, WalletConnectConfig},
};

/// Config
#[derive(uniffi::Object)]
struct Config {}
#[uniffi::export]
impl Config {
    #[uniffi::constructor]
    fn new() -> Self {
        Self {}
    }

    fn get_validators(&self) -> HashMap<String, Vec<String>> {
        get_validators()
    }

    fn get_stake_config(&self, chain: &str) -> StakeChainConfig {
        let chain = StakeChain::from_str(chain).unwrap();
        get_stake_config(chain)
    }

    fn get_swap_config(&self) -> SwapConfig {
        get_swap_config()
    }

    fn get_docs_url(&self, item: DocsUrl) -> String {
        get_docs_url(item)
    }

    fn get_social_url(&self, item: SocialUrl) -> Option<String> {
        get_social_url(item).map(|x| x.to_string())
    }

    fn get_social_url_deeplink(&self, item: SocialUrl) -> Option<String> {
        get_social_url_deeplink(item)
    }

    fn get_public_url(&self, item: PublicUrl) -> String {
        get_public_url(item).to_string()
    }

    fn get_chain_config(&self, chain: String) -> ChainConfig {
        let chain = Chain::from_str(&chain).unwrap();
        crate::chain::get_chain_config(chain)
    }

    fn get_evm_chain_config(&self, chain: String) -> EVMChainConfig {
        let chain = EVMChain::from_str(&chain).unwrap();
        get_evm_chain_config(chain)
    }

    fn get_bitcoin_chain_config(&self, chain: String) -> BitcoinChainConfig {
        let chain = BitcoinChain::from_str(&chain).unwrap();
        get_bitcoin_chain_config(chain)
    }

    fn get_wallet_connect_config(&self) -> WalletConnectConfig {
        get_wallet_connect_config()
    }

    fn get_nodes(&self) -> HashMap<String, Vec<Node>> {
        node_config::get_nodes()
    }

    fn get_nodes_for_chain(&self, chain: &str) -> Vec<Node> {
        let chain = Chain::from_str(chain).unwrap();
        node_config::get_nodes_for_chain(chain)
    }

    fn image_formatter_asset_url(&self, chain: &str, token_id: Option<String>) -> String {
        primitives::ImageFormatter::get_asset_url(ASSETS_URL, chain, token_id.as_deref())
    }

    fn image_formatter_validator_url(&self, chain: &str, id: &str) -> String {
        primitives::ImageFormatter::get_validator_url(ASSETS_URL, chain, id)
    }

    fn image_formatter_nft_asset_url(&self, url: &str, id: &str) -> String {
        primitives::ImageFormatter::get_nft_asset_url(url, id)
    }

    fn get_block_explorers(&self, chain: &str) -> Vec<String> {
        primitives::block_explorer::get_block_explorers_by_chain(chain)
            .into_iter()
            .map(|x| x.name())
            .collect()
    }

    fn get_solana_token_program(&self, id: &str) -> String {
        let id = SolanaTokenProgramId::from_str(id).unwrap();
        gem_solana::get_token_program_by_id(id).to_string()
    }

    fn get_solana_token_program_id(&self, address: &str) -> Option<String> {
        gem_solana::get_token_program_id_by_address(address).map(|x| x.as_ref().to_string())
    }
}
