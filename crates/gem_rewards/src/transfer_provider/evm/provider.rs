use crate::transfer_provider::RedemptionProvider;
use crate::{RedemptionRequest, RedemptionResult};
use alloy_primitives::hex;
use chain_traits::ChainTransactionLoad;
use gem_client::ReqwestClient;
use gem_evm::rpc::EthereumClient;
use gem_evm::signer::{create_transfer_tx, sign_eip1559_tx};
use num_traits::ToPrimitive;
use primitives::{ChainType, EVMChain, FeePriority, TransactionInputType, TransactionLoadInput, TransactionPreloadInput};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

pub struct WalletConfig {
    pub key: String,
    pub address: String,
}

pub type EvmClientProvider = Arc<dyn Fn(EVMChain) -> Option<EthereumClient<ReqwestClient>> + Send + Sync>;

pub struct EvmTransferProvider {
    wallets: HashMap<ChainType, WalletConfig>,
    client_provider: EvmClientProvider,
}

impl EvmTransferProvider {
    pub fn new(wallets: HashMap<ChainType, WalletConfig>, client_provider: EvmClientProvider) -> Self {
        Self { wallets, client_provider }
    }

    fn get_wallet(&self, chain_type: ChainType) -> Result<&WalletConfig, Box<dyn Error + Send + Sync>> {
        self.wallets
            .get(&chain_type)
            .ok_or_else(|| format!("No wallet configured for chain type {:?}", chain_type).into())
    }

    fn get_private_key(&self, wallet: &WalletConfig) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
        let key = hex::decode(wallet.key.trim_start_matches("0x"))?;
        if key.len() != 32 {
            return Err("Private key must be 32 bytes".into());
        }
        Ok(key)
    }

    fn get_client(&self, chain: EVMChain) -> Result<EthereumClient<ReqwestClient>, Box<dyn Error + Send + Sync>> {
        (self.client_provider)(chain).ok_or_else(|| format!("No client configured for chain {:?}", chain).into())
    }

    async fn execute_transfer(&self, request: RedemptionRequest) -> Result<RedemptionResult, Box<dyn Error + Send + Sync>> {
        let redemption_asset = request.asset.ok_or("Asset is required")?;
        let value = redemption_asset.value;
        let asset = redemption_asset.asset;
        let chain = asset.id.chain;
        let evm_chain = EVMChain::from_chain(chain).ok_or_else(|| format!("Chain {} is not an EVM chain", chain))?;

        let wallet = self.get_wallet(chain.chain_type())?;
        let private_key = self.get_private_key(wallet)?;
        let client = self.get_client(evm_chain)?;

        let fee_rates = client.get_transaction_fee_rates(TransactionInputType::Transfer(asset.clone())).await?;
        let fee_rate = fee_rates.iter().find(|r| r.priority == FeePriority::Normal).ok_or("No normal fee rate")?;

        let preload_input = TransactionPreloadInput {
            input_type: TransactionInputType::Transfer(asset.clone()),
            sender_address: wallet.address.clone(),
            destination_address: request.recipient_address.clone(),
        };
        let metadata = client.get_transaction_preload(preload_input).await?;

        let load_input = TransactionLoadInput {
            input_type: TransactionInputType::Transfer(asset.clone()),
            sender_address: wallet.address.clone(),
            destination_address: request.recipient_address.clone(),
            value: value.clone(),
            gas_price: fee_rate.gas_price_type.clone(),
            memo: None,
            is_max_value: false,
            metadata: metadata.clone(),
        };

        let load_data = client.get_transaction_load(load_input).await?;

        let nonce = metadata.get_sequence()? as u64;
        let chain_id: u64 = metadata.get_chain_id()?.parse()?;
        let gas_limit = load_data.fee.gas_limit.to_u64().ok_or("Gas limit overflow")?;
        let base_fee = fee_rate.gas_price_type.gas_price().to_u128().ok_or("Base fee overflow")?;
        let priority_fee = fee_rate.gas_price_type.priority_fee().to_u128().ok_or("Priority fee overflow")?;
        let max_priority_fee_per_gas = priority_fee;
        let max_fee_per_gas = base_fee + priority_fee;

        let tx = create_transfer_tx(
            &asset.id,
            &request.recipient_address,
            &value,
            nonce,
            chain_id,
            max_fee_per_gas,
            max_priority_fee_per_gas,
            gas_limit,
        )?;

        let signed_tx = sign_eip1559_tx(&tx, &private_key)?;
        let signed_tx_hex = format!("0x{}", hex::encode(&signed_tx));
        let transaction_id = client.send_raw_transaction(&signed_tx_hex).await?;

        Ok(RedemptionResult { transaction_id })
    }
}

impl RedemptionProvider for EvmTransferProvider {
    async fn process_redemption(&self, request: RedemptionRequest) -> Result<RedemptionResult, Box<dyn Error + Send + Sync>> {
        self.execute_transfer(request).await
    }
}
