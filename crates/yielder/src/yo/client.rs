use alloy_primitives::{Address, U256};
use alloy_sol_types::SolCall;
use async_trait::async_trait;
use gem_client::Client;
use gem_evm::contracts::IERC20;
use gem_evm::multicall3::IMulticall3;
use gem_evm::{jsonrpc::TransactionObject, rpc::EthereumClient};

use super::contract::{IYoGateway, IYoVaultToken};
use super::error::YieldError;
use super::model::PositionData;
use super::YoVault;

#[async_trait]
pub trait YoProvider: Send + Sync {
    fn contract_address(&self) -> Address;
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
    async fn fetch_position_data(&self, vault: YoVault, owner: Address, lookback_blocks: u64) -> Result<PositionData, YieldError>;
}

#[derive(Debug, Clone)]
pub struct YoGatewayClient<C: Client + Clone> {
    ethereum_client: EthereumClient<C>,
    contract_address: Address,
}

impl<C: Client + Clone> YoGatewayClient<C> {
    pub fn new(ethereum_client: EthereumClient<C>, contract_address: Address) -> Self {
        Self {
            ethereum_client,
            contract_address,
        }
    }

    fn deposit_call_data(yo_vault: Address, assets: U256, min_shares_out: U256, receiver: Address, partner_id: u32) -> Vec<u8> {
        IYoGateway::depositCall {
            yoVault: yo_vault,
            assets,
            minSharesOut: min_shares_out,
            receiver,
            partnerId: partner_id,
        }
        .abi_encode()
    }

    fn redeem_call_data(yo_vault: Address, shares: U256, min_assets_out: U256, receiver: Address, partner_id: u32) -> Vec<u8> {
        IYoGateway::redeemCall {
            yoVault: yo_vault,
            shares,
            minAssetsOut: min_assets_out,
            receiver,
            partnerId: partner_id,
        }
        .abi_encode()
    }
}

#[async_trait]
impl<C> YoProvider for YoGatewayClient<C>
where
    C: Client + Clone + Send + Sync + 'static,
{
    fn contract_address(&self) -> Address {
        self.contract_address
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
        let data = Self::deposit_call_data(yo_vault, assets, min_shares_out, receiver, partner_id);
        TransactionObject::new_call_with_from(&from.to_string(), &self.contract_address.to_string(), data)
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
        let data = Self::redeem_call_data(yo_vault, shares, min_assets_out, receiver, partner_id);
        TransactionObject::new_call_with_from(&from.to_string(), &self.contract_address.to_string(), data)
    }

    async fn fetch_position_data(&self, vault: YoVault, owner: Address, lookback_blocks: u64) -> Result<PositionData, YieldError> {
        let latest_block = self
            .ethereum_client
            .get_latest_block()
            .await
            .map_err(|err| YieldError::new(format!("failed to fetch latest block: {err}")))?;

        let lookback_block = latest_block.saturating_sub(lookback_blocks);
        let one_share = U256::from(10u64).pow(U256::from(vault.asset_decimals));
        let multicall_addr: Address = gem_evm::multicall3::deployment_by_chain_stack(self.ethereum_client.chain.chain_stack())
            .parse()
            .unwrap();

        let mut latest_batch = self.ethereum_client.multicall();
        let share_bal = latest_batch.add(vault.yo_token, IERC20::balanceOfCall { account: owner });
        let asset_bal = latest_batch.add(vault.asset_token, IERC20::balanceOfCall { account: owner });
        let latest_price = latest_batch.add(vault.yo_token, IYoVaultToken::convertToAssetsCall { shares: one_share });
        let latest_ts = latest_batch.add(multicall_addr, IMulticall3::getCurrentBlockTimestampCall {});

        let mut lookback_batch = self.ethereum_client.multicall();
        let lookback_price = lookback_batch.add(vault.yo_token, IYoVaultToken::convertToAssetsCall { shares: one_share });
        let lookback_ts = lookback_batch.add(multicall_addr, IMulticall3::getCurrentBlockTimestampCall {});

        let (latest, lookback) = tokio::try_join!(latest_batch.at_block(latest_block).execute(), lookback_batch.at_block(lookback_block).execute())?;

        Ok(PositionData {
            share_balance: latest.decode::<IERC20::balanceOfCall>(&share_bal)?,
            asset_balance: latest.decode::<IERC20::balanceOfCall>(&asset_bal)?,
            latest_price: latest.decode::<IYoVaultToken::convertToAssetsCall>(&latest_price)?,
            latest_timestamp: latest.decode::<IMulticall3::getCurrentBlockTimestampCall>(&latest_ts)?.to::<u64>(),
            lookback_price: lookback.decode::<IYoVaultToken::convertToAssetsCall>(&lookback_price)?,
            lookback_timestamp: lookback.decode::<IMulticall3::getCurrentBlockTimestampCall>(&lookback_ts)?.to::<u64>(),
        })
    }
}
