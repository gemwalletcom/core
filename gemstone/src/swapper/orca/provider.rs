use super::whirlpool::get_whirlpool_address;
use super::{models::*, FEE_TIER_DISCRIMINATOR, ORCA_NAME, WHIRLPOOL_CONFIG, WHIRLPOOL_PROGRAM};
use crate::network::JsonRpcResult;
use crate::{
    network::{jsonrpc::jsonrpc_call, AlienProvider},
    swapper::{models::*, GemSwapProvider, SwapperError},
};
use async_trait::async_trait;
use gem_solana::{
    jsonrpc::{Filter, Memcmp, SolanaRpc, ENCODING_BASE58},
    pubkey::Pubkey,
    WSOL_TOKEN_ADDRESS,
};
use primitives::{AssetId, Chain};
use std::{cmp::Ordering, iter::zip, str::FromStr, sync::Arc, vec};

#[derive(Debug)]
pub struct Orca {
    pub whirlpoo_program: Pubkey,
    pub whirlpool_config: Pubkey,
    pub chain: Chain,
}

impl Default for Orca {
    fn default() -> Self {
        Self {
            whirlpoo_program: Pubkey::from_str(WHIRLPOOL_PROGRAM).unwrap(),
            whirlpool_config: Pubkey::from_str(WHIRLPOOL_CONFIG).unwrap(),
            chain: Chain::Solana,
        }
    }
}

#[async_trait]
impl GemSwapProvider for Orca {
    fn name(&self) -> &'static str {
        ORCA_NAME
    }

    async fn supported_chains(&self) -> Result<Vec<Chain>, SwapperError> {
        Ok(vec![Chain::Solana])
    }

    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError> {
        if request.from_asset.chain != Chain::Solana || request.to_asset.chain != Chain::Solana {
            return Err(SwapperError::NotSupportedChain);
        }
        let from_asset = Self::get_asset_address(request.from_asset.clone())?;
        let to_asset = Self::get_asset_address(request.to_asset.clone())?;
        let pools = self
            .fetch_whirlpools(&from_asset, &to_asset, provider.clone(), request.from_asset.chain)
            .await?;
        println!("pools: {:?}", pools);

        Ok(SwapQuote {
            chain_type: request.from_asset.chain.chain_type(),
            from_value: request.value.clone(),
            to_value: "0".into(),
            provider: SwapProviderData {
                name: self.name().into(),
                routes: vec![SwapRoute {
                    route_type: "whirlpool".into(),
                    input: from_asset.to_string(),
                    output: to_asset.to_string(),
                    fee_tier: "100".to_string(),
                    gas_estimate: None,
                }],
            },
            approval: ApprovalType::None,
            request: request.clone(),
        })
    }

    async fn fetch_quote_data(&self, _quote: &SwapQuote, _provider: Arc<dyn AlienProvider>, _data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        todo!()
    }
}

impl Orca {
    pub async fn fetch_fee_tiers(&self, provider: Arc<dyn AlienProvider>) -> Result<Vec<FeeTier>, SwapperError> {
        let call = SolanaRpc::GetProgramAccounts(self.whirlpoo_program.to_string(), Self::get_program_filters());
        let response: JsonRpcResult<Vec<ProgramAccount>> = jsonrpc_call(&call, provider, &self.chain).await?;
        let result = response.extract_result()?;
        let fee_tiers = result.iter().filter_map(|x| try_borsh_decode(x.account.data[0].as_str()).ok()).collect();
        Ok(fee_tiers)
    }

    pub async fn fetch_whirlpools(
        &self,
        token_mint_1: &Pubkey,
        token_mint_2: &Pubkey,
        provider: Arc<dyn AlienProvider>,
        chain: Chain,
    ) -> Result<Vec<Whirlpool>, SwapperError> {
        let fee_tiers = self.fetch_fee_tiers(provider.clone()).await?;
        let pool_addresses = fee_tiers
            .iter()
            .filter_map(|x| self.get_pool_address(token_mint_1, token_mint_2, x.tick_spacing))
            .collect::<Vec<_>>();
        let call = SolanaRpc::GetMultipleAccounts(pool_addresses.clone());
        let response: JsonRpcResult<ValueResult<Vec<Option<AccountData>>>> = jsonrpc_call(&call, provider, &chain).await?;
        let result = response.extract_result()?.value;

        let mut pools: Vec<Whirlpool> = vec![];
        for (pool_address, account_data) in zip(pool_addresses.iter(), result.iter()) {
            println!("pool_address: {:?}, account_data: {:?}", pool_address, account_data);
            if account_data.is_none() {
                continue;
            }
            let account_data = account_data.clone().unwrap();
            let whirlpool: Whirlpool = try_borsh_decode(account_data.data[0].as_str()).map_err(|e| SwapperError::ABIError { msg: e.to_string() })?;
            pools.push(whirlpool);
        }
        Ok(pools)
    }

    fn get_pool_address(&self, token_mint_1: &Pubkey, token_mint_2: &Pubkey, tick_spacing: u16) -> Option<String> {
        let (token_mint_a, token_mint_b) = if token_mint_1.clone().to_bytes().cmp(&token_mint_2.clone().to_bytes()) == Ordering::Less {
            (token_mint_1, token_mint_2)
        } else {
            (token_mint_2, token_mint_1)
        };

        get_whirlpool_address(&self.whirlpool_config, token_mint_a, token_mint_b, tick_spacing).map(|x| x.0.to_string())
    }

    pub fn get_asset_address(asset_id: AssetId) -> Result<Pubkey, SwapperError> {
        let address = match asset_id.token_id {
            Some(token_id) => token_id,
            None => WSOL_TOKEN_ADDRESS.to_string(),
        };
        Pubkey::from_str(&address).map_err(|_| SwapperError::InvalidAddress { address })
    }

    pub fn get_program_filters() -> Vec<Filter> {
        vec![
            Filter {
                memcmp: Memcmp {
                    offset: 0,
                    bytes: FEE_TIER_DISCRIMINATOR.into(),
                    encoding: ENCODING_BASE58.into(),
                },
            },
            Filter {
                memcmp: Memcmp {
                    offset: 8,
                    bytes: WHIRLPOOL_CONFIG.into(),
                    encoding: ENCODING_BASE58.into(),
                },
            },
        ]
    }
}
