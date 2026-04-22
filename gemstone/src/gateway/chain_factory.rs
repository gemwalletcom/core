use super::preferences::PreferencesWrapper;
use super::{GatewayError, GemPreferences};
use crate::alien::{AlienProvider, new_alien_client};
use crate::network::JsonRpcClient;
use chain_traits::ChainTraits;
use gem_algorand::rpc::AlgorandClientIndexer;
use gem_algorand::rpc::client::AlgorandClient;
use gem_aptos::rpc::client::AptosClient;
use gem_bitcoin::rpc::client::BitcoinClient;
use gem_cardano::rpc::client::CardanoClient;
use gem_cosmos::rpc::client::CosmosClient;
use gem_evm::rpc::EthereumClient;
use gem_hypercore::rpc::client::HyperCoreClient;
use gem_near::rpc::client::NearClient;
use gem_polkadot::rpc::client::PolkadotClient;
use gem_solana::rpc::client::SolanaClient;
use gem_stellar::rpc::client::StellarClient;
use gem_sui::rpc::client::SuiClient;
use gem_ton::rpc::client::TonClient;
use gem_tron::rpc::{client::TronClient, trongrid::client::TronGridClient};
use gem_xrp::rpc::client::XRPClient;
use primitives::{BitcoinChain, Chain, EVMChain, chain_cosmos::CosmosChain};
use std::sync::Arc;

pub struct ChainClientFactory {
    alien: Arc<dyn AlienProvider>,
    preferences: Arc<dyn GemPreferences>,
    secure_preferences: Arc<dyn GemPreferences>,
}

impl ChainClientFactory {
    pub fn new(alien: Arc<dyn AlienProvider>, preferences: Arc<dyn GemPreferences>, secure_preferences: Arc<dyn GemPreferences>) -> Self {
        Self {
            alien,
            preferences,
            secure_preferences,
        }
    }

    pub async fn create(&self, chain: Chain) -> Result<Arc<dyn ChainTraits>, GatewayError> {
        let url = self.alien.get_endpoint(chain).map_err(|e| GatewayError::PlatformError { msg: e.to_string() })?;
        self.create_with_url(chain, url).await
    }

    pub async fn create_with_url(&self, chain: Chain, url: String) -> Result<Arc<dyn ChainTraits>, GatewayError> {
        let alien_client = new_alien_client(url, self.alien.clone());
        match chain {
            Chain::HyperCore => {
                let preferences = Arc::new(PreferencesWrapper {
                    preferences: self.preferences.clone(),
                });
                let secure_preferences = Arc::new(PreferencesWrapper {
                    preferences: self.secure_preferences.clone(),
                });
                Ok(Arc::new(HyperCoreClient::new_with_preferences(alien_client, preferences, secure_preferences)))
            }
            Chain::Bitcoin | Chain::BitcoinCash | Chain::Litecoin | Chain::Doge | Chain::Zcash => {
                Ok(Arc::new(BitcoinClient::new(alien_client, BitcoinChain::from_chain(chain).unwrap())))
            }
            Chain::Cardano => Ok(Arc::new(CardanoClient::new(alien_client))),
            Chain::Stellar => Ok(Arc::new(StellarClient::new(alien_client))),
            Chain::Sui => Ok(Arc::new(SuiClient::new(JsonRpcClient::new(alien_client.clone())))),
            Chain::Xrp => Ok(Arc::new(XRPClient::new(JsonRpcClient::new(alien_client.clone())))),
            Chain::Algorand => Ok(Arc::new(AlgorandClient::new(alien_client.clone(), AlgorandClientIndexer::new(alien_client.clone())))),
            Chain::Near => Ok(Arc::new(NearClient::new(JsonRpcClient::new(alien_client.clone())))),
            Chain::Aptos => Ok(Arc::new(AptosClient::new(alien_client))),
            Chain::Cosmos | Chain::Osmosis | Chain::Celestia | Chain::Thorchain | Chain::Injective | Chain::Sei | Chain::Noble => {
                Ok(Arc::new(CosmosClient::new(CosmosChain::from_chain(chain).unwrap(), alien_client)))
            }
            Chain::Ton => Ok(Arc::new(TonClient::new(alien_client))),
            Chain::Tron => Ok(Arc::new(TronClient::new(alien_client.clone(), TronGridClient::new(alien_client.clone(), String::new())))),
            Chain::Polkadot => Ok(Arc::new(PolkadotClient::new(alien_client))),
            Chain::Solana => Ok(Arc::new(SolanaClient::new(JsonRpcClient::new(alien_client.clone())))),
            Chain::Ethereum
            | Chain::Arbitrum
            | Chain::SmartChain
            | Chain::Polygon
            | Chain::Optimism
            | Chain::Base
            | Chain::AvalancheC
            | Chain::OpBNB
            | Chain::Fantom
            | Chain::Gnosis
            | Chain::Manta
            | Chain::Blast
            | Chain::ZkSync
            | Chain::Linea
            | Chain::Mantle
            | Chain::Celo
            | Chain::World
            | Chain::Sonic
            | Chain::SeiEvm
            | Chain::Abstract
            | Chain::Berachain
            | Chain::Ink
            | Chain::Unichain
            | Chain::Hyperliquid
            | Chain::Plasma
            | Chain::Monad
            | Chain::XLayer
            | Chain::Stable => Ok(Arc::new(EthereumClient::new(JsonRpcClient::new(alien_client), EVMChain::from_chain(chain).unwrap()))),
        }
    }
}
