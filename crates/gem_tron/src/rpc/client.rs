use async_trait::async_trait;
use chain_traits::{ChainAccount, ChainPerpetual, ChainTraits};
use num_bigint::BigUint;
use primitives::{asset_type::AssetType, chain::Chain, Asset, AssetId};
use std::{error::Error, str::FromStr};

use crate::models::{
    Block, BlockTransactions, BlockTransactionsInfo, ChainParameter, ChainParametersResponse, Transaction, TransactionReceiptData,
    TriggerConstantContractRequest, TriggerConstantContractResponse, TronTransactionBroadcast, WitnessesList,
};
use crate::models::{
    TronAccount, TronAccountRequest, TronAccountUsage, TronBlock, TronEmptyAccount, TronReward, TronSmartContractCall, TronSmartContractResult,
};
use crate::rpc::constants::{DECIMALS_SELECTOR, DEFAULT_OWNER_ADDRESS, NAME_SELECTOR, SYMBOL_SELECTOR};
use crate::rpc::trongrid::client::TronGridClient;
use gem_client::Client;
use gem_evm::erc20::{decode_abi_string, decode_abi_uint8};

#[derive(Clone)]
pub struct TronClient<C: Client> {
    pub client: C,
    pub trongrid_client: TronGridClient<C>,
}

impl<C: Client> TronClient<C> {
    pub fn new(client: C, trongrid_client: TronGridClient<C>) -> Self {
        Self { client, trongrid_client }
    }

    pub async fn get_block(&self) -> Result<Block, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get("/wallet/getblock").await?)
    }

    pub async fn get_block_tranactions(&self, block: u64) -> Result<BlockTransactions, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/wallet/getblockbynum?num={}", block)).await?)
    }

    pub async fn get_block_tranactions_reciepts(&self, block: u64) -> Result<BlockTransactionsInfo, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/wallet/gettransactioninfobyblocknum?num={}", block)).await?)
    }

    pub async fn get_transaction(&self, id: String) -> Result<Transaction, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/wallet/gettransactionbyid?value={}", id)).await?)
    }

    pub async fn get_transaction_reciept(&self, id: String) -> Result<TransactionReceiptData, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/wallet/gettransactioninfobyid?value={}", id)).await?)
    }

    pub async fn trigger_constant_contract(
        &self,
        contract_address: &str,
        function_selector: &str,
        parameter: &str,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        self.trigger_constant_contract_with_owner(DEFAULT_OWNER_ADDRESS, contract_address, function_selector, parameter)
            .await
    }

    pub async fn trigger_constant_contract_with_owner(
        &self,
        owner_address: &str,
        contract_address: &str,
        function_selector: &str,
        parameter: &str,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let request_payload = TriggerConstantContractRequest {
            owner_address: owner_address.to_owned(),
            contract_address: contract_address.to_string(),
            function_selector: function_selector.to_string(),
            parameter: parameter.to_string(),
            visible: true,
        };

        let response: TriggerConstantContractResponse = self.client.post("/wallet/triggerconstantcontract", &request_payload, None).await?;

        if response.constant_result.is_empty() {
            return Err("Empty response from Tron contract call".into());
        }

        Ok(response.constant_result[0].clone())
    }
}

impl<C: Client> TronClient<C> {
    pub fn get_chain(&self) -> Chain {
        Chain::Tron
    }

    pub async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        Ok(self.get_block().await?.block_header.raw_data.number)
    }

    pub async fn get_witnesses_list(&self) -> Result<WitnessesList, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get("/wallet/listwitnesses").await?)
    }

    pub async fn get_chain_parameters(&self) -> Result<Vec<ChainParameter>, Box<dyn Error + Send + Sync>> {
        let response: ChainParametersResponse = self.client.get("/wallet/getchainparameters").await?;
        Ok(response.chain_parameter)
    }

    pub async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        let name = self.trigger_constant_contract(&token_id, NAME_SELECTOR, "").await?;
        let symbol = self.trigger_constant_contract(&token_id, SYMBOL_SELECTOR, "").await?;
        let decimals = self.trigger_constant_contract(&token_id, DECIMALS_SELECTOR, "").await?;

        let name = decode_abi_string(&name)?;
        let symbol = decode_abi_string(&symbol)?;
        let decimals = decode_abi_uint8(&decimals)?;
        let asset_id = AssetId::from(Chain::Tron, Some(token_id.clone()));
        Ok(Asset::new(asset_id, name, symbol, decimals as i32, AssetType::TRC20))
    }

    pub async fn get_account(&self, address: &str) -> Result<TronAccount, Box<dyn Error + Send + Sync>> {
        let request = TronAccountRequest {
            address: address.to_string(),
            visible: true,
        };

        Ok(self.client.post("/wallet/getaccount", &request, None).await?)
    }

    pub async fn get_account_usage(&self, address: &str) -> Result<TronAccountUsage, Box<dyn Error + Send + Sync>> {
        let request = TronAccountRequest {
            address: address.to_string(),
            visible: true,
        };

        Ok(self.client.post("/wallet/getaccountresource", &request, None).await?)
    }

    pub async fn get_reward(&self, address: &str) -> Result<TronReward, Box<dyn Error + Send + Sync>> {
        let request = TronAccountRequest {
            address: address.to_string(),
            visible: true,
        };

        Ok(self.client.post("/wallet/getReward", &request, None).await?)
    }

    pub async fn trigger_smart_contract(&self, request: &TronSmartContractCall) -> Result<TronSmartContractResult, Box<dyn Error + Send + Sync>> {
        Ok(self.client.post("/wallet/triggerconstantcontract", request, None).await?)
    }

    pub async fn is_new_account(&self, address: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let request = TronAccountRequest {
            address: address.to_string(),
            visible: true,
        };

        let account: TronEmptyAccount = self.client.post("/wallet/getaccount", &request, None).await?;
        Ok(account.address.is_none_or(|addr| addr.is_empty()))
    }

    pub async fn broadcast_transaction(&self, data: String) -> Result<TronTransactionBroadcast, Box<dyn Error + Send + Sync>> {
        let json_value: serde_json::Value = serde_json::from_str(&data)?;
        Ok(self.client.post("/wallet/broadcasttransaction", &json_value, None).await?)
    }

    pub async fn get_tron_block(&self) -> Result<TronBlock, Box<dyn Error + Send + Sync>> {
        Ok(self.client.post("/wallet/getnowblock", &serde_json::json!({}), None).await?)
    }

    pub async fn estimate_trc20_transfer_gas(
        &self,
        sender_address: String,
        contract_address: String,
        recipient_address: String,
        value: String,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let value_bigint = BigUint::from_str(&value).map_err(|e| format!("Failed to parse value as decimal: {}", e))?;
        let value_hex = format!("{:0>64}", hex::encode(value_bigint.to_bytes_be()));
        let parameter = format!("{}{}", recipient_address, value_hex);

        let request_payload = TriggerConstantContractRequest {
            owner_address: sender_address,
            contract_address,
            function_selector: "transfer(address,uint256)".to_string(),
            parameter,
            visible: true,
        };

        let response: TriggerConstantContractResponse = self.client.post("/wallet/triggerconstantcontract", &request_payload, None).await?;

        Ok(response.energy_used.to_string())
    }
}

// Trait implementations required for gateway integration
#[async_trait]
impl<C: Client + Clone> ChainTraits for TronClient<C> {}

#[async_trait]
impl<C: Client + Clone> ChainAccount for TronClient<C> {}

#[async_trait]
impl<C: Client + Clone> ChainPerpetual for TronClient<C> {}

impl<C: Client + Clone> chain_traits::ChainProvider for TronClient<C> {
    fn get_chain(&self) -> primitives::Chain {
        Chain::Tron
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_encoding_for_trc20_transfer() {
        let value = "1000000".to_string(); // 1 USDT (6 decimals)
        let recipient_address = "0000000000000000000000003e1451cdb84d440345de6195b0384d1b77aa4eaa".to_string();

        let value_bigint = BigUint::from_str(&value).unwrap();
        let value_hex = format!("{:0>64}", hex::encode(value_bigint.to_bytes_be()));
        let parameter = format!("{}{}", recipient_address, value_hex);

        // For 1000000 (decimal), the hex should be f4240 padded to 64 chars
        assert_eq!(value_hex, "00000000000000000000000000000000000000000000000000000000000f4240");
        assert_eq!(
            parameter,
            "0000000000000000000000003e1451cdb84d440345de6195b0384d1b77aa4eaa00000000000000000000000000000000000000000000000000000000000f4240"
        );
    }

    #[test]
    fn test_large_value_encoding() {
        let value = "16777216".to_string(); // Large value that was causing issues

        let value_bigint = BigUint::from_str(&value).unwrap();
        let value_hex = format!("{:0>64}", hex::encode(value_bigint.to_bytes_be()));

        // 16777216 decimal = 0x1000000 hex
        assert_eq!(value_hex, "0000000000000000000000000000000000000000000000000000000001000000");
    }
}
