use alloy_primitives::{Address, U256, hex};
use alloy_sol_types::SolCall;
use async_trait::async_trait;
use gem_client::Client;
use gem_evm::{jsonrpc::TransactionObject, rpc::EthereumClient};
use primitives::Chain;
use serde_json::json;

use super::{YO_GATEWAY_BASE_MAINNET, YO_PARTNER_ID_GEM, YoVault, contract::IYoGateway, error::YieldError};

alloy_sol_types::sol! {
    interface IYoVaultToken {
        function convertToAssets(uint256 shares) external view returns (uint256 assets);
    }
}

#[async_trait]
pub trait YoGatewayApi: Send + Sync {
    fn contract_address(&self) -> Address;
    fn chain(&self) -> Chain;
    fn build_deposit_transaction(
        &self,
        from: Address,
        yo_vault: Address,
        assets: U256,
        min_shares_out: U256,
        receiver: Address,
        partner_id: u32,
    ) -> TransactionObject;
    fn build_redeem_transaction(
        &self,
        from: Address,
        yo_vault: Address,
        shares: U256,
        min_assets_out: U256,
        receiver: Address,
        partner_id: u32,
    ) -> TransactionObject;
    async fn balance_of(&self, token: Address, owner: Address) -> Result<U256, YieldError>;
    async fn convert_to_assets_at_block(&self, yo_vault: Address, shares: U256, block_number: u64) -> Result<U256, YieldError>;
    async fn latest_block_number(&self) -> Result<u64, YieldError>;
    async fn block_timestamp(&self, block_number: u64) -> Result<u64, YieldError>;
}

#[derive(Debug, Clone)]
pub struct YoGatewayClient<C: Client + Clone> {
    ethereum_client: EthereumClient<C>,
    contract_address: Address,
}

impl<C: Client + Clone> YoGatewayClient<C> {
    pub const fn default_partner_id() -> u32 {
        YO_PARTNER_ID_GEM
    }

    pub fn new(ethereum_client: EthereumClient<C>, contract_address: Address) -> Self {
        Self {
            ethereum_client,
            contract_address,
        }
    }

    pub fn base_mainnet(ethereum_client: EthereumClient<C>) -> Self {
        Self::new(ethereum_client, YO_GATEWAY_BASE_MAINNET)
    }

    pub fn contract_address(&self) -> Address {
        self.contract_address
    }

    pub async fn quote_convert_to_shares(&self, yo_vault: Address, assets: U256) -> Result<U256, YieldError> {
        self.call_gateway_contract(IYoGateway::quoteConvertToSharesCall { yoVault: yo_vault, assets })
            .await
    }

    pub async fn quote_convert_to_assets(&self, yo_vault: Address, shares: U256) -> Result<U256, YieldError> {
        self.call_gateway_contract(IYoGateway::quoteConvertToAssetsCall { yoVault: yo_vault, shares })
            .await
    }

    pub async fn quote_preview_deposit(&self, yo_vault: Address, assets: U256) -> Result<U256, YieldError> {
        self.call_gateway_contract(IYoGateway::quotePreviewDepositCall { yoVault: yo_vault, assets })
            .await
    }

    pub async fn quote_preview_redeem(&self, yo_vault: Address, shares: U256) -> Result<U256, YieldError> {
        self.call_gateway_contract(IYoGateway::quotePreviewRedeemCall { yoVault: yo_vault, shares })
            .await
    }

    pub async fn get_asset_allowance(&self, yo_vault: Address, owner: Address) -> Result<U256, YieldError> {
        self.call_gateway_contract(IYoGateway::getAssetAllowanceCall { yoVault: yo_vault, owner }).await
    }

    pub async fn get_share_allowance(&self, yo_vault: Address, owner: Address) -> Result<U256, YieldError> {
        self.call_gateway_contract(IYoGateway::getShareAllowanceCall { yoVault: yo_vault, owner }).await
    }

    pub async fn quote_convert_to_shares_for(&self, vault: YoVault, assets: U256) -> Result<U256, YieldError> {
        self.quote_convert_to_shares(vault.yo_token, assets).await
    }

    pub async fn quote_convert_to_assets_for(&self, vault: YoVault, shares: U256) -> Result<U256, YieldError> {
        self.quote_convert_to_assets(vault.yo_token, shares).await
    }

    pub async fn quote_preview_deposit_for(&self, vault: YoVault, assets: U256) -> Result<U256, YieldError> {
        self.quote_preview_deposit(vault.yo_token, assets).await
    }

    pub async fn quote_preview_redeem_for(&self, vault: YoVault, shares: U256) -> Result<U256, YieldError> {
        self.quote_preview_redeem(vault.yo_token, shares).await
    }

    pub async fn get_asset_allowance_for(&self, vault: YoVault, owner: Address) -> Result<U256, YieldError> {
        self.get_asset_allowance(vault.yo_token, owner).await
    }

    pub async fn get_share_allowance_for(&self, vault: YoVault, owner: Address) -> Result<U256, YieldError> {
        self.get_share_allowance(vault.yo_token, owner).await
    }

    pub fn deposit_call_data(yo_vault: Address, assets: U256, min_shares_out: U256, receiver: Address, partner_id: u32) -> Vec<u8> {
        IYoGateway::depositCall {
            yoVault: yo_vault,
            assets,
            minSharesOut: min_shares_out,
            receiver,
            partnerId: partner_id,
        }
        .abi_encode()
    }

    pub fn redeem_call_data(yo_vault: Address, shares: U256, min_assets_out: U256, receiver: Address, partner_id: u32) -> Vec<u8> {
        IYoGateway::redeemCall {
            yoVault: yo_vault,
            shares,
            minAssetsOut: min_assets_out,
            receiver,
            partnerId: partner_id,
        }
        .abi_encode()
    }

    pub fn deposit_call_data_for(vault: YoVault, assets: U256, min_shares_out: U256, receiver: Address, partner_id: u32) -> Vec<u8> {
        Self::deposit_call_data(vault.yo_token, assets, min_shares_out, receiver, partner_id)
    }

    pub fn redeem_call_data_for(vault: YoVault, shares: U256, min_assets_out: U256, receiver: Address, partner_id: u32) -> Vec<u8> {
        Self::redeem_call_data(vault.yo_token, shares, min_assets_out, receiver, partner_id)
    }

    pub fn build_deposit_transaction(
        &self,
        from: Address,
        yo_vault: Address,
        assets: U256,
        min_shares_out: U256,
        receiver: Address,
        partner_id: u32,
    ) -> TransactionObject {
        let data = Self::deposit_call_data(yo_vault, assets, min_shares_out, receiver, partner_id);
        TransactionObject::new_call_with_from(&from.to_string(), &self.contract_address.to_string(), data)
    }

    pub fn build_redeem_transaction(
        &self,
        from: Address,
        yo_vault: Address,
        shares: U256,
        min_assets_out: U256,
        receiver: Address,
        partner_id: u32,
    ) -> TransactionObject {
        let data = Self::redeem_call_data(yo_vault, shares, min_assets_out, receiver, partner_id);
        TransactionObject::new_call_with_from(&from.to_string(), &self.contract_address.to_string(), data)
    }

    async fn call_gateway_contract<Call>(&self, call: Call) -> Result<Call::Return, YieldError>
    where
        Call: SolCall,
    {
        self.call_contract_at_block(call, self.contract_address, None).await
    }

    async fn call_contract_at_block<Call>(&self, call: Call, contract: Address, block_number: Option<u64>) -> Result<Call::Return, YieldError>
    where
        Call: SolCall,
    {
        let payload = hex::encode_prefixed(call.abi_encode());
        let contract_address = contract.to_string();

        let block_param = block_number
            .map(|number| format!("0x{number:x}"))
            .map_or_else(|| json!("latest"), serde_json::Value::String);

        let response: String = self
            .ethereum_client
            .client
            .call(
                "eth_call",
                json!([
                    {
                        "to": contract_address,
                        "data": payload,
                    },
                    block_param
                ]),
            )
            .await
            .map_err(|err| YieldError::new(format!("yo gateway rpc call failed: {err}")))?;

        if response.trim().is_empty() || response == "0x" {
            return Err(YieldError::new("yo gateway response did not contain data"));
        }

        let decoded = hex::decode(&response).map_err(|err| YieldError::new(format!("invalid hex returned by yo gateway: {err}")))?;
        Call::abi_decode_returns(&decoded).map_err(|err| YieldError::new(format!("failed to decode yo gateway response: {err}")))
    }
}

#[async_trait]
impl<C> YoGatewayApi for YoGatewayClient<C>
where
    C: Client + Clone + Send + Sync + 'static,
{
    fn contract_address(&self) -> Address {
        self.contract_address
    }

    fn chain(&self) -> Chain {
        self.ethereum_client.get_chain()
    }

    fn build_deposit_transaction(
        &self,
        from: Address,
        yo_vault: Address,
        assets: U256,
        min_shares_out: U256,
        receiver: Address,
        partner_id: u32,
    ) -> TransactionObject {
        <YoGatewayClient<C>>::build_deposit_transaction(self, from, yo_vault, assets, min_shares_out, receiver, partner_id)
    }

    fn build_redeem_transaction(
        &self,
        from: Address,
        yo_vault: Address,
        shares: U256,
        min_assets_out: U256,
        receiver: Address,
        partner_id: u32,
    ) -> TransactionObject {
        <YoGatewayClient<C>>::build_redeem_transaction(self, from, yo_vault, shares, min_assets_out, receiver, partner_id)
    }

    async fn balance_of(&self, token: Address, owner: Address) -> Result<U256, YieldError> {
        alloy_sol_types::sol! {
            interface IERC20Balance {
                function balanceOf(address account) external view returns (uint256);
            }
        }

        let call = IERC20Balance::balanceOfCall { account: owner }.abi_encode();
        let payload = hex::encode_prefixed(call);
        let params = json!([
            {
                "to": token.to_string(),
                "data": payload,
            },
            "latest"
        ]);

        let result: String = self
            .ethereum_client
            .client
            .call("eth_call", params)
            .await
            .map_err(|err| YieldError::new(format!("yo gateway rpc call failed: {err}")))?;

        let value = result.trim_start_matches("0x");
        U256::from_str_radix(value, 16).map_err(|err| YieldError::new(format!("invalid balance data: {err}")))
    }

    async fn convert_to_assets_at_block(&self, yo_vault: Address, shares: U256, block_number: u64) -> Result<U256, YieldError> {
        self.call_contract_at_block(IYoVaultToken::convertToAssetsCall { shares }, yo_vault, Some(block_number))
            .await
    }

    async fn latest_block_number(&self) -> Result<u64, YieldError> {
        self.ethereum_client
            .get_latest_block()
            .await
            .map_err(|err| YieldError::new(format!("yo gateway failed to fetch latest block: {err}")))
    }

    async fn block_timestamp(&self, block_number: u64) -> Result<u64, YieldError> {
        let block_hex = format!("0x{block_number:x}");
        let mut blocks = self
            .ethereum_client
            .get_blocks(&[block_hex], false)
            .await
            .map_err(|err| YieldError::new(format!("yo gateway failed to fetch block {block_number}: {err}")))?;

        let block = blocks
            .pop()
            .ok_or_else(|| YieldError::new(format!("yo gateway missing block data for {block_number}")))?;

        block
            .timestamp
            .to_string()
            .parse::<u64>()
            .map_err(|err| YieldError::new(format!("yo gateway failed to parse timestamp for block {block_number}: {err}")))
    }
}
