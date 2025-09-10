use std::error::Error;

use crate::models::account::Balances;
use crate::models::staking::{Delegations, Rewards, UnbondingDelegations};
use crate::models::{Account, AccountResponse, BroadcastRequest, BroadcastResponse, InjectiveAccount};
use crate::models::{
    AnnualProvisionsResponse, BlockResponse, InflationResponse, OsmosisEpochProvisionsResponse, OsmosisMintParamsResponse, StakingPoolResponse, SupplyResponse,
    TransactionResponse, TransactionsResponse, ValidatorsResponse,
};
use chain_traits::{ChainAccount, ChainAddressStatus, ChainPerpetual, ChainTraits};
use gem_client::Client;
use primitives::chain_cosmos::CosmosChain;

pub struct CosmosClient<C: Client> {
    chain: CosmosChain,
    pub client: C,
}

impl<C: Client> CosmosClient<C> {
    pub fn new(chain: CosmosChain, client: C) -> Self {
        Self { chain, client }
    }

    pub fn get_chain(&self) -> CosmosChain {
        self.chain
    }

    pub fn get_amount(&self, coins: Vec<cosmos_sdk_proto::cosmos::base::v1beta1::Coin>) -> Option<String> {
        Some(
            coins
                .into_iter()
                .filter(|x| x.denom == self.chain.as_chain().as_denom().unwrap_or_default())
                .collect::<Vec<_>>()
                .first()?
                .amount
                .clone(),
        )
    }

    pub async fn get_transaction(&self, hash: String) -> Result<TransactionResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/cosmos/tx/v1beta1/txs/{}", hash)).await?)
    }

    pub async fn get_block(&self, block: &str) -> Result<BlockResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/cosmos/base/tendermint/v1beta1/blocks/{}", block)).await?)
    }

    pub async fn get_transactions_by_address_with_limit(&self, address: &str, limit: usize) -> Result<Vec<TransactionResponse>, Box<dyn Error + Send + Sync>> {
        let query_name = match self.chain {
            CosmosChain::Cosmos => Some("query"),
            CosmosChain::Osmosis => Some("query"),
            CosmosChain::Celestia => Some("events"),
            CosmosChain::Thorchain => None,
            CosmosChain::Injective => Some("query"),
            CosmosChain::Sei => Some("events"),
            CosmosChain::Noble => Some("query"),
        };
        if query_name.is_none() {
            return Ok(vec![]);
        }
        let query_name = query_name.unwrap();

        let inbound = self
            .get_transactions_by_query(query_name, &format!("message.sender='{address}'"), limit)
            .await?;
        let outbound = self
            .get_transactions_by_query(query_name, &format!("message.recipient='{address}'"), limit)
            .await?;
        let responses = inbound.tx_responses.into_iter().chain(outbound.tx_responses.into_iter()).collect::<Vec<_>>();
        let txs = inbound.txs.into_iter().chain(outbound.txs.into_iter()).collect::<Vec<_>>();
        Ok(responses
            .into_iter()
            .zip(txs)
            .map(|(response, tx)| TransactionResponse { tx, tx_response: response })
            .collect::<Vec<_>>())
    }

    pub async fn get_transactions_by_query(&self, query_name: &str, query: &str, limit: usize) -> Result<TransactionsResponse, Box<dyn Error + Send + Sync>> {
        let url = format!("/cosmos/tx/v1beta1/txs?{}={}&pagination.limit={}&page=1", query_name, query, limit);
        Ok(self.client.get(&url).await?)
    }

    pub async fn get_validators(&self) -> Result<ValidatorsResponse, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .get("/cosmos/staking/v1beta1/validators?status=BOND_STATUS_BONDED&pagination.limit=100")
            .await?)
    }

    pub async fn get_staking_pool(&self) -> Result<StakingPoolResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get("/cosmos/staking/v1beta1/pool").await?)
    }

    pub async fn get_inflation(&self) -> Result<InflationResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get("/cosmos/mint/v1beta1/inflation").await?)
    }

    pub async fn get_annual_provisions(&self) -> Result<AnnualProvisionsResponse, Box<dyn Error + Send + Sync>> {
        let url = "/cosmos/mint/v1beta1/annual_provisions";
        Ok(self.client.get(url).await?)
    }

    pub async fn get_supply_by_denom(&self, denom: &str) -> Result<SupplyResponse, Box<dyn Error + Send + Sync>> {
        let url = format!("/cosmos/bank/v1beta1/supply/by_denom?denom={}", denom);
        Ok(self.client.get(&url).await?)
    }

    pub async fn get_osmosis_mint_params(&self) -> Result<OsmosisMintParamsResponse, Box<dyn Error + Send + Sync>> {
        let url = "/osmosis/mint/v1beta1/params";
        Ok(self.client.get(url).await?)
    }

    pub async fn get_osmosis_epoch_provisions(&self) -> Result<OsmosisEpochProvisionsResponse, Box<dyn Error + Send + Sync>> {
        let url = "/osmosis/mint/v1beta1/epoch_provisions";
        Ok(self.client.get(url).await?)
    }

    pub async fn get_balances(&self, address: &str) -> Result<Balances, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/cosmos/bank/v1beta1/balances/{}", address)).await?)
    }

    pub async fn get_delegations(&self, address: &str) -> Result<Delegations, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/cosmos/staking/v1beta1/delegations/{}", address)).await?)
    }

    pub async fn get_unbonding_delegations(&self, address: &str) -> Result<UnbondingDelegations, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .get(&format!("/cosmos/staking/v1beta1/delegators/{}/unbonding_delegations", address))
            .await?)
    }

    pub async fn get_delegation_rewards(&self, address: &str) -> Result<Rewards, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/cosmos/distribution/v1beta1/delegators/{}/rewards", address)).await?)
    }

    pub fn get_base_fee(&self) -> u64 {
        crate::constants::get_base_fee(self.chain)
    }

    pub async fn get_account_info(&self, address: &str) -> Result<Account, Box<dyn Error + Send + Sync>> {
        let url = format!("/cosmos/auth/v1beta1/accounts/{}", address);
        match self.chain {
            CosmosChain::Injective => {
                let response: AccountResponse<InjectiveAccount> = self.client.get(&url).await?;
                Ok(response.account.base_account)
            }
            _ => {
                let response: AccountResponse<Account> = self.client.get(&url).await?;
                Ok(response.account)
            }
        }
    }

    pub async fn get_node_info(&self) -> Result<crate::models::NodeInfoResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get("/cosmos/base/tendermint/v1beta1/node_info").await?)
    }

    pub async fn broadcast_transaction(&self, data: &str) -> Result<BroadcastResponse, Box<dyn Error + Send + Sync>> {
        let request: BroadcastRequest = serde_json::from_str(data)?;
        Ok(self.client.post("/cosmos/tx/v1beta1/txs", &request, None).await?)
    }
}

impl<C: Client> ChainAccount for CosmosClient<C> {}

impl<C: Client> ChainPerpetual for CosmosClient<C> {}

impl<C: Client> ChainAddressStatus for CosmosClient<C> {}

impl<C: Client> ChainTraits for CosmosClient<C> {}

impl<C: Client> chain_traits::ChainProvider for CosmosClient<C> {
    fn get_chain(&self) -> primitives::Chain {
        self.chain.as_chain()
    }
}
