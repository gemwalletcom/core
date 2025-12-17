use crate::transfer_provider::RedemptionProvider;
use crate::{RedemptionRequest, RedemptionResult};
use alloy_primitives::hex;
use chain_traits::ChainTransactionLoad;
use gem_client::ReqwestClient;
use gem_evm::rpc::EthereumClient;
use gem_evm::signer::{create_transfer_tx, sign_eip1559_tx};
use num_traits::ToPrimitive;
use primitives::{
    Asset, AssetId, AssetType, ChainType, EVMChain, FeePriority, TransactionInputType, TransactionLoadInput, TransactionLoadMetadata, TransactionPreloadInput,
};
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
        let asset = request.asset.ok_or("Asset is required")?;
        let chain = asset.asset_id.chain;
        let evm_chain = EVMChain::from_chain(chain).ok_or_else(|| format!("Chain {} is not an EVM chain", chain))?;

        let wallet = self.get_wallet(chain.chain_type())?;
        let private_key = self.get_private_key(wallet)?;
        let client = self.get_client(evm_chain)?;

        let transfer_asset = asset_from_id(&asset.asset_id);
        let fee_rates = client.get_transaction_fee_rates(TransactionInputType::Transfer(transfer_asset.clone())).await?;
        let fee_rate = fee_rates.iter().find(|r| r.priority == FeePriority::Normal).ok_or("No normal fee rate")?;

        let load_input = TransactionLoadInput {
            input_type: TransactionInputType::Transfer(transfer_asset.clone()),
            sender_address: wallet.address.clone(),
            destination_address: request.recipient_address.clone(),
            value: asset.amount.clone(),
            gas_price: fee_rate.gas_price_type.clone(),
            memo: None,
            is_max_value: false,
            metadata: TransactionLoadMetadata::None,
        };

        let load_data = client.get_transaction_load(load_input).await?;

        let preload_input = TransactionPreloadInput {
            input_type: TransactionInputType::Transfer(transfer_asset),
            sender_address: wallet.address.clone(),
            destination_address: request.recipient_address.clone(),
        };
        let metadata = client.get_transaction_preload(preload_input).await?;

        let nonce = metadata.get_sequence()? as u64;
        let chain_id: u64 = metadata.get_chain_id()?.parse()?;
        let gas_limit = load_data.fee.gas_limit.to_u64().ok_or("Gas limit overflow")?;
        let base_fee = fee_rate.gas_price_type.gas_price().to_u128().ok_or("Base fee overflow")?;
        let priority_fee = fee_rate.gas_price_type.priority_fee().to_u128().ok_or("Priority fee overflow")?;
        let max_priority_fee_per_gas = priority_fee;
        let max_fee_per_gas = base_fee + priority_fee;

        let tx = create_transfer_tx(
            &asset.asset_id,
            &request.recipient_address,
            &asset.amount,
            nonce,
            chain_id,
            max_fee_per_gas,
            max_priority_fee_per_gas,
            gas_limit,
        )?;

        let signed_tx = sign_eip1559_tx(&tx, &private_key)?;
        let signed_tx_hex = format!("0x{}", hex::encode(&signed_tx));

        let tx_hash = client.send_raw_transaction(&signed_tx_hex).await?;

        Ok(RedemptionResult { transaction_id: tx_hash })
    }
}

fn asset_from_id(asset_id: &AssetId) -> Asset {
    let asset_type = if asset_id.is_token() { AssetType::ERC20 } else { AssetType::NATIVE };
    Asset::new(asset_id.clone(), String::new(), String::new(), 0, asset_type)
}

impl RedemptionProvider for EvmTransferProvider {
    async fn process_redemption(&self, request: RedemptionRequest) -> Result<RedemptionResult, Box<dyn Error + Send + Sync>> {
        self.execute_transfer(request).await
    }
}
