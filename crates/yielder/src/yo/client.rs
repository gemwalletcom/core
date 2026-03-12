use alloy_primitives::{Address, U256};
use alloy_sol_types::SolCall;
use async_trait::async_trait;
use gem_client::Client;
use gem_evm::contracts::IERC20;
use gem_evm::jsonrpc::TransactionObject;
use gem_evm::multicall3::{create_call3, decode_call3_return};
use gem_evm::rpc::EthereumClient;
use primitives::swap::ApprovalData;

use super::assets::YoAsset;
use super::contract::{IYoGateway, IYoVaultToken};
use crate::error::YielderError;

#[derive(Debug, Clone)]
pub struct PositionData {
    pub share_balance: U256,
    pub asset_balance: U256,
}

#[async_trait]
pub trait YoClient: Send + Sync {
    fn build_deposit_transaction(&self, from: Address, yo_token: Address, assets: U256, min_shares_out: U256, receiver: Address, partner_id: u32) -> TransactionObject;
    fn build_redeem_transaction(&self, from: Address, yo_token: Address, shares: U256, min_assets_out: U256, receiver: Address, partner_id: u32) -> TransactionObject;
    async fn get_positions_batch(&self, assets: &[YoAsset], owner: Address) -> Result<Vec<PositionData>, YielderError>;
    async fn check_token_allowance(&self, token: Address, owner: Address, amount: U256) -> Result<Option<ApprovalData>, YielderError>;
    async fn convert_to_shares(&self, yo_token: Address, assets: U256) -> Result<U256, YielderError>;
}

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
}

#[async_trait]
impl<C> YoClient for YoGatewayClient<C>
where
    C: Client + Clone + Send + Sync + 'static,
{
    fn build_deposit_transaction(&self, from: Address, yo_token: Address, assets: U256, min_shares_out: U256, receiver: Address, partner_id: u32) -> TransactionObject {
        let data = IYoGateway::depositCall {
            yoVault: yo_token,
            assets,
            minSharesOut: min_shares_out,
            receiver,
            partnerId: partner_id,
        }
        .abi_encode();
        TransactionObject::new_call_with_from(&from.to_string(), &self.contract_address.to_string(), data)
    }

    fn build_redeem_transaction(&self, from: Address, yo_token: Address, shares: U256, min_assets_out: U256, receiver: Address, partner_id: u32) -> TransactionObject {
        let data = IYoGateway::redeemCall {
            yoVault: yo_token,
            shares,
            minAssetsOut: min_assets_out,
            receiver,
            partnerId: partner_id,
        }
        .abi_encode();
        TransactionObject::new_call_with_from(&from.to_string(), &self.contract_address.to_string(), data)
    }

    async fn get_positions_batch(&self, assets: &[YoAsset], owner: Address) -> Result<Vec<PositionData>, YielderError> {
        let calls: Vec<_> = assets
            .iter()
            .flat_map(|a| {
                let vault = a.yo_token.to_string();
                [
                    create_call3(&vault, IYoVaultToken::balanceOfCall { account: owner }),
                    create_call3(&vault, IYoVaultToken::totalAssetsCall {}),
                    create_call3(&vault, IYoVaultToken::totalSupplyCall {}),
                ]
            })
            .collect();
        let results = self.ethereum_client.multicall3(calls).await?;

        results
            .chunks(3)
            .map(|chunk| {
                let shares = decode_call3_return::<IYoVaultToken::balanceOfCall>(&chunk[0])?;
                let total_assets = decode_call3_return::<IYoVaultToken::totalAssetsCall>(&chunk[1])?;
                let total_supply = decode_call3_return::<IYoVaultToken::totalSupplyCall>(&chunk[2])?;
                let asset_balance = convert_to_assets_ceil(shares, total_assets, total_supply);
                Ok(PositionData {
                    share_balance: shares,
                    asset_balance,
                })
            })
            .collect()
    }

    async fn check_token_allowance(&self, token: Address, owner: Address, amount: U256) -> Result<Option<ApprovalData>, YielderError> {
        let spender = self.contract_address;

        let calls = vec![create_call3(&token.to_string(), IERC20::allowanceCall { owner, spender })];
        let results = self.ethereum_client.multicall3(calls).await?;
        let allowance = decode_call3_return::<IERC20::allowanceCall>(&results[0])?;

        if allowance < amount {
            Ok(Some(ApprovalData {
                token: token.to_string(),
                spender: spender.to_string(),
                value: amount.to_string(),
            }))
        } else {
            Ok(None)
        }
    }

    async fn convert_to_shares(&self, yo_token: Address, assets: U256) -> Result<U256, YielderError> {
        let gateway = self.contract_address.to_string();
        let calls = vec![create_call3(&gateway, IYoGateway::quoteConvertToSharesCall { yoVault: yo_token, assets })];
        let results = self.ethereum_client.multicall3(calls).await?;
        Ok(decode_call3_return::<IYoGateway::quoteConvertToSharesCall>(&results[0])?)
    }
}

/// ERC4626 ceiling division: rounds up instead of down so display matches deposited amount.
fn convert_to_assets_ceil(shares: U256, total_assets: U256, total_supply: U256) -> U256 {
    if shares.is_zero() || total_supply.is_zero() {
        return U256::ZERO;
    }
    (shares * total_assets + total_supply - U256::from(1)) / total_supply
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_to_assets_ceil() {
        assert_eq!(convert_to_assets_ceil(U256::from(100), U256::from(1000), U256::from(500)), U256::from(200));
        assert_eq!(convert_to_assets_ceil(U256::from(1), U256::from(1000), U256::from(500)), U256::from(2));
        assert_eq!(convert_to_assets_ceil(U256::from(500), U256::from(1000), U256::from(500)), U256::from(1000));
        assert_eq!(convert_to_assets_ceil(U256::from(1), U256::from(10), U256::from(3)), U256::from(4));
        assert_eq!(convert_to_assets_ceil(U256::from(2), U256::from(10), U256::from(3)), U256::from(7));
        assert_eq!(convert_to_assets_ceil(U256::ZERO, U256::from(1000), U256::from(500)), U256::ZERO);
        assert_eq!(convert_to_assets_ceil(U256::from(100), U256::from(1000), U256::ZERO), U256::ZERO);
        assert_eq!(convert_to_assets_ceil(U256::ZERO, U256::ZERO, U256::ZERO), U256::ZERO);
    }
}
