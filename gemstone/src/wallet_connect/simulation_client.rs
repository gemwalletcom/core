use std::sync::Arc;

use ::simulation::evm::SimulationClient;
use gem_evm::rpc::EthereumClient;
use gem_wallet_connect::{SignDigestType as WcSignDigestType, WalletConnectTransactionType as WcWalletConnectTransactionType};
use primitives::{Chain, EVMChain, SimulationResult};

use crate::{
    GemstoneError,
    alien::{AlienClient, AlienProvider, new_alien_client},
    message::sign_type::SignDigestType,
    network::JsonRpcClient,
};

use super::{WalletConnectTransactionType, simulation};

#[derive(uniffi::Object)]
pub struct WalletConnectSimulationClient {
    provider: Arc<dyn AlienProvider>,
}

#[uniffi::export]
impl WalletConnectSimulationClient {
    #[uniffi::constructor]
    pub fn new(provider: Arc<dyn AlienProvider>) -> Self {
        Self { provider }
    }

    pub async fn simulate_sign_message(&self, chain: Chain, sign_type: SignDigestType, data: String, session_domain: String) -> Result<SimulationResult, GemstoneError> {
        let sign_type: WcSignDigestType = sign_type.into();
        let validation_warnings = simulation::sign_message_validation_warnings(chain, &sign_type, &data, &session_domain);

        let simulation = match sign_type {
            WcSignDigestType::Eip712 => match simulation::parse_eip712_message(&data) {
                Some(message) => self.simulate_eip712_message(chain, &message).await?,
                None => SimulationResult::default(),
            },
            _ => SimulationResult::default(),
        };

        Ok(simulation.prepend_warnings(validation_warnings))
    }

    pub async fn simulate_send_transaction(&self, chain: Chain, transaction_type: WalletConnectTransactionType, data: String) -> Result<SimulationResult, GemstoneError> {
        let transaction_type: WcWalletConnectTransactionType = transaction_type.into();
        let validation_warnings = simulation::send_transaction_validation_warnings(&transaction_type, &data);

        let simulation = match transaction_type {
            WcWalletConnectTransactionType::Ethereum => self.simulate_ethereum_transaction(chain, &data).await?,
            _ => SimulationResult::default(),
        };

        Ok(simulation.prepend_warnings(validation_warnings))
    }
}

impl WalletConnectSimulationClient {
    async fn simulate_eip712_message(&self, chain: Chain, message: &gem_evm::eip712::EIP712Message) -> Result<SimulationResult, GemstoneError> {
        let client = self.ethereum_client(chain).ok_or("No RPC client available")?;
        Ok(SimulationClient::new(&client).simulate_eip712_message(chain, message).await?)
    }

    async fn simulate_ethereum_transaction(&self, chain: Chain, data: &str) -> Result<SimulationResult, GemstoneError> {
        let (transaction, bytes) = simulation::decode_ethereum_calldata(data).ok_or("Failed to decode transaction")?;
        let client = self.ethereum_client(chain).ok_or("No RPC client available")?;
        Ok(SimulationClient::new(&client).simulate_evm_calldata(chain, &bytes, &transaction.to).await?)
    }

    fn ethereum_client(&self, chain: Chain) -> Option<EthereumClient<AlienClient>> {
        let chain = EVMChain::from_chain(chain)?;
        let url = self.provider.get_endpoint(chain.to_chain()).ok()?;
        let client = new_alien_client(url, self.provider.clone());
        Some(EthereumClient::new(JsonRpcClient::new(client), chain))
    }
}
