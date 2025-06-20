use primitives::{asset_type::AssetType, chain::Chain, Asset, AssetId};
use std::error::Error;

use super::model::{Block, BlockTransactions, BlockTransactionsInfo, TriggerConstantContractRequest, TriggerConstantContractResponse};
use crate::rpc::constants::{DECIMALS_SELECTOR, DEFAULT_OWNER_ADDRESS, NAME_SELECTOR, SYMBOL_SELECTOR};
use crate::rpc::model::TransactionReceiptData;
use gem_evm::erc20::{decode_abi_string, decode_abi_uint8};
use reqwest_middleware::ClientWithMiddleware;

#[derive(Debug, Clone)]
pub struct TronClient {
    url: String,
    client: ClientWithMiddleware,
}

impl TronClient {
    pub fn new(client: ClientWithMiddleware, url: String) -> Self {
        Self { url, client }
    }

    pub async fn get_block(&self) -> Result<Block, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(format!("{}/wallet/getblock", self.url)).send().await?.json().await?)
    }

    pub async fn get_block_tranactions(&self, block: i64) -> Result<BlockTransactions, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .get(format!("{}/walletsolidity/getblockbynum?num={}", self.url, block))
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn get_block_tranactions_reciepts(&self, block: i64) -> Result<BlockTransactionsInfo, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .get(format!("{}/walletsolidity/gettransactioninfobyblocknum?num={}", self.url, block))
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn get_transaction_reciept(&self, id: String) -> Result<TransactionReceiptData, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .get(format!("{}/walletsolidity/gettransactioninfobyid?value={}", self.url, id))
            .send()
            .await?
            .json()
            .await?)
    }

    async fn trigger_constant_contract(&self, contract_address: &str, function_selector: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let request_payload = TriggerConstantContractRequest {
            owner_address: DEFAULT_OWNER_ADDRESS.to_owned(),
            contract_address: contract_address.to_string(),
            function_selector: function_selector.to_string(),
            parameter: "".to_string(),
            visible: true,
        };

        let response = self
            .client
            .post(format!("{}/wallet/triggerconstantcontract", self.url))
            .json(&request_payload)
            .send()
            .await?;
        Ok(response.json::<TriggerConstantContractResponse>().await?.constant_result[0].clone())
    }
}

impl TronClient {
    pub fn get_chain(&self) -> Chain {
        Chain::Tron
    }

    pub async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        Ok(self.get_block().await?.block_header.raw_data.number)
    }

    pub async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        let name = self.trigger_constant_contract(&token_id, NAME_SELECTOR).await?;
        let symbol = self.trigger_constant_contract(&token_id, SYMBOL_SELECTOR).await?;
        let decimals = self.trigger_constant_contract(&token_id, DECIMALS_SELECTOR).await?;

        let name = decode_abi_string(&name)?;
        let symbol = decode_abi_string(&symbol)?;
        let decimals = decode_abi_uint8(&decimals)?;
        let asset_id = AssetId::from(Chain::Tron, Some(token_id.clone()));
        Ok(Asset::new(asset_id, name, symbol, decimals as i32, AssetType::TRC20))
    }
}
