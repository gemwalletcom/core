use super::fee_tiers::get_fee_tiers;
use super::whirlpool::{get_tick_array_address, get_whirlpool_address};
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
use orca_whirlpools_core::{
    get_tick_array_start_tick_index, swap_quote_by_input_token, TickArrayFacade, TickArrays, TickFacade, WhirlpoolFacade, WhirlpoolRewardInfoFacade,
    TICK_ARRAY_SIZE,
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
        let amount_in = request.value.parse::<u64>().map_err(|_| SwapperError::InvalidAmount)?;
        let options = request.options.clone().unwrap_or_default();
        let slippage_bps = options.slippage_bps as u16;

        let from_asset = Self::get_asset_address(request.from_asset.clone())?;
        let to_asset = Self::get_asset_address(request.to_asset.clone())?;
        let mut pools = self
            .fetch_whirlpools(&from_asset, &to_asset, provider.clone(), request.from_asset.chain)
            .await?;

        if pools.is_empty() {
            return Err(SwapperError::NoQuoteAvailable);
        }

        // sort by liquidity ↓ and fee_rate ↑
        pools.sort_by(|(_, a), (_, b)| b.liquidity.cmp(&a.liquidity).then(a.fee_rate.cmp(&b.fee_rate)));
        let (pool_address, pool) = pools.first().unwrap();
        let pool_address = Pubkey::from_str(pool_address).unwrap();
        println!("first_pool: {:?}", pool);
        println!("pool_address: {:?}", pool_address);

        let _token_accounts = self.fetch_token_accounts(&pool.token_mint_a, &pool.token_mint_b, provider.clone()).await?;
        let tick_arrays = self.fetch_tick_arrays(&pool_address, pool, provider.clone()).await?;

        let quote = swap_quote_by_input_token(amount_in, true, slippage_bps, pool.into(), tick_arrays, None, None).map_err(|c| SwapperError::NetworkError {
            msg: format!("swap_quote_by_input_token error: {:?}", c),
        })?;

        Ok(SwapQuote {
            chain_type: request.from_asset.chain.chain_type(),
            from_value: request.value.clone(),
            to_value: quote.token_est_out.to_string(),
            provider: SwapProviderData {
                name: self.name().into(),
                routes: vec![SwapRoute {
                    route_type: "whirlpool".into(),
                    input: from_asset.to_string(),
                    output: to_asset.to_string(),
                    fee_tier: pool.fee_rate.to_string(),
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
    #[allow(unused)]
    pub async fn fetch_fee_tiers(&self, provider: Arc<dyn AlienProvider>) -> Result<Vec<FeeTier>, SwapperError> {
        let call = SolanaRpc::GetProgramAccounts(self.whirlpoo_program.to_string(), Self::get_program_filters());
        let response: JsonRpcResult<Vec<ProgramAccount>> = jsonrpc_call(&call, provider, &self.chain).await?;
        let result = response.extract_result()?;
        let fee_tiers: Vec<FeeTier> = result.iter().filter_map(|x| try_borsh_decode(x.account.data[0].as_str()).ok()).collect();
        Ok(fee_tiers)
    }

    pub async fn fetch_whirlpools(
        &self,
        token_mint_1: &Pubkey,
        token_mint_2: &Pubkey,
        provider: Arc<dyn AlienProvider>,
        chain: Chain,
    ) -> Result<Vec<(String, Whirlpool)>, SwapperError> {
        let fee_tiers = get_fee_tiers();
        let pool_addresses = fee_tiers
            .iter()
            .filter_map(|x| self.get_pool_address(token_mint_1, token_mint_2, x.tick_spacing))
            .collect::<Vec<_>>();
        let call = SolanaRpc::GetMultipleAccounts(pool_addresses.clone());
        let response: JsonRpcResult<ValueResult<Vec<Option<AccountData>>>> = jsonrpc_call(&call, provider, &chain).await?;
        let result = response.extract_result()?.value;

        let mut pools: Vec<(String, Whirlpool)> = vec![];
        for (pool_address, account_data) in zip(pool_addresses.iter(), result.iter()) {
            if account_data.is_none() {
                continue;
            }
            let account_data = account_data.clone().unwrap();
            let whirlpool: Whirlpool = try_borsh_decode(account_data.data[0].as_str()).map_err(|e| SwapperError::ABIError { msg: e.to_string() })?;
            pools.push((pool_address.to_string(), whirlpool));
        }
        Ok(pools)
    }

    pub async fn fetch_tick_arrays(&self, pool_address: &Pubkey, pool: &Whirlpool, provider: Arc<dyn AlienProvider>) -> Result<TickArrays, SwapperError> {
        let start_index = get_tick_array_start_tick_index(pool.tick_current_index, pool.tick_spacing);
        let offset = (pool.tick_spacing as i32) * (TICK_ARRAY_SIZE as i32);
        let tick_arrays = [
            start_index,
            start_index + offset,
            start_index + 2 * offset,
            start_index - offset,
            start_index - 2 * offset,
        ];
        println!("tick_arrays: {:?}", tick_arrays);
        let tick_addresses: Vec<String> = tick_arrays
            .iter()
            .map(|x| get_tick_array_address(pool_address, *x))
            .filter_map(|x| match x {
                Some(x) => Some(x.0.to_string()),
                None => None,
            })
            .collect();
        println!("tick_addresses: {:?}", tick_addresses);

        let call = SolanaRpc::GetMultipleAccounts(tick_addresses);
        let response: JsonRpcResult<ValueResult<Vec<AccountData>>> = jsonrpc_call(&call, provider, &self.chain).await?;
        let tick_accounts = response.extract_result()?.value;

        println!("tick_accounts: {:?}", tick_accounts.len());

        let mut tick_arrays: Vec<TickArrayFacade> = vec![];

        for tick_account in tick_accounts.iter() {
            let tick = try_borsh_decode::<TickArray>(tick_account.data[0].as_str()).map_err(|e| SwapperError::ABIError { msg: e.to_string() })?;
            tick_arrays.push(TickArrayFacade::from(&tick));
        }

        println!("tick_arrays: {:?}", tick_arrays.len());

        if tick_arrays.is_empty() {
            return Err(SwapperError::ABIError {
                msg: "fetch_tick_arrays error".into(),
            });
        }
        Ok(tick_arrays[0].into())
        // let result: [TickArrayFacade; 5] = std::array::from_fn(|i| TickArrayFacade::from(&tick_arrays[i]));
        // Ok(result.into())
    }

    pub async fn fetch_token_accounts(
        &self,
        token_mint_a: &Pubkey,
        token_mint_b: &Pubkey,
        provider: Arc<dyn AlienProvider>,
    ) -> Result<Vec<AccountData>, SwapperError> {
        let rpc = SolanaRpc::GetMultipleAccounts(vec![token_mint_a.to_string(), token_mint_b.to_string()]);
        let response: JsonRpcResult<ValueResult<Vec<AccountData>>> = jsonrpc_call(&rpc, provider.clone(), &self.chain).await?;
        let token_accounts = response.extract_result()?.value;
        Ok(token_accounts)
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

impl From<&Whirlpool> for WhirlpoolFacade {
    fn from(val: &Whirlpool) -> Self {
        Self {
            tick_spacing: val.tick_spacing,
            fee_rate: val.fee_rate,
            protocol_fee_rate: val.protocol_fee_rate,
            liquidity: val.liquidity,
            sqrt_price: val.sqrt_price,
            tick_current_index: val.tick_current_index,
            fee_growth_global_a: val.fee_growth_global_a,
            fee_growth_global_b: val.fee_growth_global_b,
            reward_last_updated_timestamp: val.reward_last_updated_timestamp,
            reward_infos: std::array::from_fn(|i| WhirlpoolRewardInfoFacade::from(&val.reward_infos[i])),
        }
    }
}

impl From<&WhirlpoolRewardInfo> for WhirlpoolRewardInfoFacade {
    fn from(val: &WhirlpoolRewardInfo) -> Self {
        Self {
            emissions_per_second_x64: val.emissions_per_second_x64,
            growth_global_x64: val.growth_global_x64,
        }
    }
}

impl From<&TickArray> for TickArrayFacade {
    fn from(value: &TickArray) -> Self {
        Self {
            start_tick_index: value.start_tick_index,
            ticks: std::array::from_fn(|i| TickFacade::from(&value.ticks[i])),
        }
    }
}

impl From<&Tick> for TickFacade {
    fn from(value: &Tick) -> Self {
        Self {
            initialized: value.initialized,
            liquidity_net: value.liquidity_net,
            liquidity_gross: value.liquidity_gross,
            fee_growth_outside_a: value.fee_growth_outside_a,
            fee_growth_outside_b: value.fee_growth_outside_b,
            reward_growths_outside: value.reward_growths_outside,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_tick_array_start_tick_index() {
        let tick_current_index = -15865;
        let tick_spacing = 4;
        let start_index = get_tick_array_start_tick_index(tick_current_index, tick_spacing);

        assert_eq!(start_index, -16192);

        let pool = Pubkey::from_str("Czfq3xZZDmsdGdUyrNLtRhGc47cXcZtLG4crryfu44zE").unwrap();
        let tick_array_address = get_tick_array_address(&pool, start_index).unwrap();

        assert_eq!(tick_array_address.0.to_string(), "3M9oTcoC5viBCNuJEKgwCrQDEbE3Rh6CpTGP5C2jGHzU");
    }

    #[test]
    fn test_decode_tick_array() {
        let data = include_str!("test/tick_array.json");

        let response: JsonRpcResult<ValueResult<Vec<AccountData>>> = serde_json::from_slice(data.as_bytes()).unwrap();
        let tick_accounts = response.extract_result().unwrap().value;

        let tick_arrays = tick_accounts
            .iter()
            .filter_map(|x| try_borsh_decode::<TickArray>(x.data[0].as_str()).ok())
            .collect::<Vec<_>>();

        assert_eq!(tick_arrays.len(), 5);
    }
}
