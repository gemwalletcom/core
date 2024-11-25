use super::fee_tiers::get_splash_pool_fee_tiers;
use super::whirlpool::{get_tick_array_address, get_whirlpool_address};
use super::{models::*, FEE_TIER_DISCRIMINATOR, WHIRLPOOL_CONFIG, WHIRLPOOL_PROGRAM};
use crate::network::JsonRpcResult;
use crate::{
    network::{jsonrpc::jsonrpc_call, AlienProvider},
    swapper::{models::*, GemSwapProvider, SwapperError},
};
use async_trait::async_trait;
use gem_solana::{
    get_asset_address,
    jsonrpc::{Filter, Memcmp, SolanaRpc, ENCODING_BASE58},
    pubkey::Pubkey,
};
use orca_whirlpools_core::{
    get_tick_array_start_tick_index, swap_quote_by_input_token, TickArrayFacade, TickArrays, TickFacade, WhirlpoolFacade, WhirlpoolRewardInfoFacade,
    TICK_ARRAY_SIZE,
};
use primitives::Chain;
use std::{cmp::Ordering, iter::zip, str::FromStr, sync::Arc, vec};

#[derive(Debug)]
pub struct Orca {
    pub whirlpool_program: Pubkey,
    pub whirlpool_config: Pubkey,
    pub chain: Chain,
}

impl Default for Orca {
    fn default() -> Self {
        Self {
            whirlpool_program: Pubkey::from_str(WHIRLPOOL_PROGRAM).unwrap(),
            whirlpool_config: Pubkey::from_str(WHIRLPOOL_CONFIG).unwrap(),
            chain: Chain::Solana,
        }
    }
}

#[async_trait]
impl GemSwapProvider for Orca {
    fn provider(&self) -> SwapProvider {
        SwapProvider::Orca
    }

    fn supported_chains(&self) -> Vec<Chain> {
        vec![Chain::Solana]
    }

    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError> {
        if request.from_asset.chain != Chain::Solana || request.to_asset.chain != Chain::Solana {
            return Err(SwapperError::NotSupportedChain);
        }

        let amount_in = request.value.parse::<u64>().map_err(|_| SwapperError::InvalidAmount)?;
        let options = request.options.clone().unwrap_or_default();
        let slippage_bps = options.slippage_bps as u16;

        let from_asset = get_asset_address(&request.from_asset).ok_or_else(|| SwapperError::InvalidAddress {
            address: request.from_asset.to_string(),
        })?;
        let to_asset = get_asset_address(&request.to_asset).ok_or_else(|| SwapperError::InvalidAddress {
            address: request.from_asset.to_string(),
        })?;
        let fee_tiers = self.fetch_fee_tiers(provider.clone()).await?;
        let mut pools = self
            .fetch_whirlpools(&from_asset, &to_asset, fee_tiers, provider.clone(), request.from_asset.chain)
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

        // let _token_accounts = self.fetch_token_accounts(&pool.token_mint_a, &pool.token_mint_b, provider.clone()).await?;
        let tick_array = self.fetch_tick_arrays(&pool_address, pool, provider.clone()).await?;

        let tick_array_facades = tick_array.into_iter().map(|x| TickArrayFacade::from(&x)).collect::<Vec<_>>();
        let result: [TickArrayFacade; 5] = std::array::from_fn(|i| tick_array_facades[i]);
        let tick_arrays = TickArrays::from(result);

        let quote = swap_quote_by_input_token(amount_in, true, slippage_bps, pool.into(), tick_arrays, None, None).map_err(|c| SwapperError::NetworkError {
            msg: format!("swap_quote_by_input_token error: {:?}", c),
        })?;

        Ok(SwapQuote {
            from_value: request.value.clone(),
            to_value: quote.token_est_out.to_string(),
            data: SwapProviderData {
                provider: self.provider(),
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

    async fn get_transaction_status(&self, _chain: Chain, _transaction_hash: &str, _provider: Arc<dyn AlienProvider>) -> Result<bool, SwapperError> {
        // TODO: the transaction status from the RPC
        Ok(true)
    }
}

impl Orca {
    #[allow(unused)]
    pub async fn fetch_fee_tiers(&self, provider: Arc<dyn AlienProvider>) -> Result<Vec<FeeTier>, SwapperError> {
        let call = SolanaRpc::GetProgramAccounts(self.whirlpool_program.to_string(), Self::get_program_filters());
        let response: JsonRpcResult<Vec<ProgramAccount>> = jsonrpc_call(&call, provider, &self.chain).await?;
        let result = response.extract_result()?;
        let fee_tiers: Vec<FeeTier> = result.iter().filter_map(|x| try_borsh_decode(x.account.data[0].as_str()).ok()).collect();
        Ok(fee_tiers)
    }

    #[allow(unused)]
    pub async fn fetch_splash_pool(
        &self,
        token_mint_1: &Pubkey,
        token_mint_2: &Pubkey,
        provider: Arc<dyn AlienProvider>,
        chain: Chain,
    ) -> Result<Whirlpool, SwapperError> {
        let fee_tiers = vec![get_splash_pool_fee_tiers()];
        let result = self.fetch_whirlpools(token_mint_1, token_mint_2, fee_tiers, provider, chain).await?;
        let pool = result.first().ok_or(SwapperError::NotSupportedPair)?;
        Ok(pool.1.clone())
    }

    pub async fn fetch_whirlpools(
        &self,
        token_mint_1: &Pubkey,
        token_mint_2: &Pubkey,
        fee_tiers: Vec<FeeTier>,
        provider: Arc<dyn AlienProvider>,
        chain: Chain,
    ) -> Result<Vec<(String, Whirlpool)>, SwapperError> {
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

    pub async fn fetch_tick_arrays(&self, pool_address: &Pubkey, pool: &Whirlpool, provider: Arc<dyn AlienProvider>) -> Result<Vec<TickArray>, SwapperError> {
        let start_index = get_tick_array_start_tick_index(pool.tick_current_index, pool.tick_spacing);
        let offset = (pool.tick_spacing as i32) * (TICK_ARRAY_SIZE as i32);
        let tick_arrays = [
            start_index - 2 * offset,
            start_index - offset,
            start_index,
            start_index + offset,
            start_index + 2 * offset,
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
        let base64_strs: Vec<String> = tick_accounts.iter().map(|x| x.data[0].clone()).collect();
        let mut tick_array: Vec<TickArray> = vec![];
        for base64_str in base64_strs.iter() {
            let tick = try_borsh_decode::<TickArray>(base64_str).unwrap();
            tick_array.push(tick);
        }

        Ok(tick_array)
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
    fn test_swap_quote_by_input_token() -> Result<(), SwapperError> {
        let data = include_str!("test/tick_array_response.json");
        let response: JsonRpcResult<ValueResult<Vec<AccountData>>> = serde_json::from_slice(data.as_bytes()).unwrap();
        let tick_accounts = response.extract_result().unwrap().value;
        let base64_strs: Vec<String> = tick_accounts.iter().map(|x| x.data[0].clone()).collect();
        let mut tick_array: Vec<TickArray> = vec![];
        for base64_str in base64_strs.iter() {
            let tick: TickArray = try_borsh_decode(base64_str).unwrap();
            tick_array.push(tick);
        }

        tick_array.sort_by_key(|x| x.start_tick_index);

        let tick_array_facades = tick_array.into_iter().map(|x| TickArrayFacade::from(&x)).collect::<Vec<_>>();

        let result: [TickArrayFacade; 5] = std::array::from_fn(|i| tick_array_facades[i]);
        let tick_arrays = TickArrays::from(result);

        let amount_in = 1000000;
        let slippage_bps = 100;
        let base64_str = "P5XRDOGAYwkT5EH4ORPKaLBjT7Al/eqohzfoQRDRJV41ezN33e4czf8EAAQAkAEBACUn6rOx9gIAAAAAAAAAAADZ0q3a01wPfgAAAAAAAAAApsj///QCNRYAAAAA7MHhBAAAAAAGm4hX/quBhPtof2NGGMA12sQ53BrrO1WYoPAAAAAAAchN8kM4mDvkqFswl7r0C8lXEQjSiawAs2jfF11Edc96okZrwvdXv2MAAAAAAAAAAMb6evO+2606PWXzaqvJdDGxu+TC0vbg5HymAgNFL11hFl+VcsWpaqUC3VEQVKJqbSWO98HW1sGu4SkZFNxRAjLtNOmyVWgdCwAAAAAAAAAAaZY8ZwAAAAAMANCv64YU2n8Zq6AtQPGMaSWF9lAg387T1eX5qcDE4Q8bkJQIzrVDfhKReyB9qZTQ6FenQB4SLAPfa/fG1/wqvR0xrxfe/zwmhIFgCsr+SxQJjA/hQbf0oc34STRkRAMAAAAAAAAAAAAAAAAAAAAAIxHh3tFPDkQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAC9HTGvF97/PCaEgWAKyv5LFAmMD+FBt/ShzfhJNGREAwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAL0dMa8X3v88JoSBYArK/ksUCYwP4UG39KHN+Ek0ZEQDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
        let pool: Whirlpool = try_borsh_decode(base64_str).unwrap();

        let quote =
            swap_quote_by_input_token(amount_in, true, slippage_bps, (&pool).into(), tick_arrays, None, None).map_err(|c| SwapperError::ComputeQuoteError {
                msg: format!("swap_quote_by_input_token error: {:?}", c),
            })?;
        assert_eq!(quote.token_min_out, 239958);
        Ok(())
    }

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
}
