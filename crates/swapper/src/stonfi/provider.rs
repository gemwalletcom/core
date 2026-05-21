use super::{
    client::StonfiClient,
    constants::{FALLBACK_ROUTERS, RouterInfo},
    model::{QuotePath, SwapSimulation},
    quote::{DiscoveredPool, apply_slippage, compute_amount_out, router_model, scaled_next_min_ask_amount, static_candidates, token_address},
    tx_builder::{NextSwapParams, ReferralParams, SwapTransactionParams, build_swap_transaction},
};
use crate::{
    FetchQuoteData, ProviderData, ProviderType, Quote, QuoteRequest, Route, RpcClient, RpcProvider, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperQuoteAsset,
    SwapperQuoteData,
    fees::{ReferralFee, default_referral_fees, quote_value_after_reserve_by_chain},
    route_cache::DiscoveryCache,
};
use async_trait::async_trait;
use futures::future::join_all;
use gem_client::Client;
use gem_ton::{address::Address, rpc::client::TonClient};
use num_bigint::BigUint;
use primitives::{AssetId, Chain, asset_constants::TON_USDT_ASSET_ID};
use std::{fmt::Debug, str::FromStr, sync::Arc};

#[derive(Debug)]
pub struct Stonfi<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    provider: ProviderType,
    client: StonfiClient<C>,
    route_cache: DiscoveryCache<DiscoveredPool, String>,
}

impl Stonfi<RpcClient> {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        let endpoint = rpc_provider.get_endpoint(Chain::Ton).expect("failed to get TON endpoint for STON.fi");
        Self::new_with_client(TonClient::new(RpcClient::new(endpoint, rpc_provider)))
    }
}

impl<C> Stonfi<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    pub fn new_with_client(ton_client: TonClient<C>) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::StonfiV2),
            client: StonfiClient::new(ton_client),
            route_cache: DiscoveryCache::default(),
        }
    }

    fn intermediary_tokens() -> [SwapperQuoteAsset; 2] {
        [SwapperQuoteAsset::from(AssetId::from_chain(Chain::Ton)), SwapperQuoteAsset::from(TON_USDT_ASSET_ID.clone())]
    }

    async fn quote_path_via_intermediary(
        &self,
        intermediary: &SwapperQuoteAsset,
        from_value: &str,
        request: &QuoteRequest,
        allow_discovery: bool,
    ) -> Result<QuotePath, SwapperError> {
        if !self.should_quote_intermediary_path(&request.from_asset, intermediary, &request.to_asset, allow_discovery) {
            return Err(SwapperError::NoQuoteAvailable);
        }

        let to_intermediary = self
            .quote_swap(&request.from_asset, from_value, intermediary, request.options.slippage.bps, allow_discovery, true)
            .await?;
        if !to_intermediary.router.is_supported_v2() {
            return Err(SwapperError::InvalidRoute);
        }

        let from_intermediary = self
            .quote_swap(
                intermediary,
                &to_intermediary.ask_units,
                &request.to_asset,
                request.options.slippage.bps,
                allow_discovery,
                true,
            )
            .await?;
        if !from_intermediary.router.is_supported_v2() {
            return Err(SwapperError::InvalidRoute);
        }

        Ok(QuotePath {
            to_value: from_intermediary.ask_units.clone(),
            routes: vec![
                Route {
                    input: request.from_asset.asset_id(),
                    output: intermediary.asset_id(),
                    route_data: serde_json::to_string(&to_intermediary)?,
                },
                Route {
                    input: intermediary.asset_id(),
                    output: request.to_asset.asset_id(),
                    route_data: serde_json::to_string(&from_intermediary)?,
                },
            ],
        })
    }

    async fn quote_direct(&self, request: &QuoteRequest, from_value: &str, allow_discovery: bool) -> Result<QuotePath, SwapperError> {
        let simulation = self
            .quote_swap(&request.from_asset, from_value, &request.to_asset, request.options.slippage.bps, allow_discovery, false)
            .await?;
        Ok(QuotePath {
            to_value: simulation.ask_units.clone(),
            routes: vec![Route {
                input: request.from_asset.asset_id(),
                output: request.to_asset.asset_id(),
                route_data: serde_json::to_string(&simulation)?,
            }],
        })
    }

    async fn quote_intermediary_paths(&self, request: &QuoteRequest, from_value: &str, allow_discovery: bool) -> Vec<Result<QuotePath, SwapperError>> {
        let mut paths = Vec::new();
        for intermediary in Self::intermediary_tokens().into_iter().filter(|x| {
            let intermediary_id = x.asset_id();
            intermediary_id != request.from_asset.asset_id() && intermediary_id != request.to_asset.asset_id()
        }) {
            let path = self.quote_path_via_intermediary(&intermediary, from_value, request, allow_discovery).await;
            let is_ok = path.is_ok();
            paths.push(path);
            if allow_discovery && is_ok {
                break;
            }
        }
        paths
    }

    fn has_known_candidates(&self, from_asset: &SwapperQuoteAsset, to_asset: &SwapperQuoteAsset, require_v2: bool) -> bool {
        let from_token = token_address(from_asset);
        let to_token = token_address(to_asset);
        let (cached_candidates, _) = self.route_cache.get(&from_token, &to_token);
        self.route_cache
            .get_route(&from_token, &to_token)
            .is_some_and(|route| route_has_supported_version(&route, require_v2))
            || route_has_supported_version(&cached_candidates, require_v2)
            || static_candidates(&from_token, &to_token)
                .iter()
                .any(|candidate| candidate_has_supported_version(candidate, require_v2))
    }

    fn should_quote_intermediary_path(&self, from_asset: &SwapperQuoteAsset, intermediary: &SwapperQuoteAsset, to_asset: &SwapperQuoteAsset, allow_discovery: bool) -> bool {
        let first_known = self.has_known_candidates(from_asset, intermediary, true);
        let second_known = self.has_known_candidates(intermediary, to_asset, true);
        if allow_discovery { first_known || second_known } else { first_known && second_known }
    }

    fn referral_fee() -> ReferralFee {
        default_referral_fees().ton
    }

    async fn quote_swap(
        &self,
        from_asset: &SwapperQuoteAsset,
        from_value: &str,
        to_asset: &SwapperQuoteAsset,
        slippage_bps: u32,
        allow_discovery: bool,
        require_v2: bool,
    ) -> Result<SwapSimulation, SwapperError> {
        let from_token = token_address(from_asset);
        let to_token = token_address(to_asset);
        let amount = BigUint::from_str(from_value)?;
        if amount == BigUint::from(0u8) {
            return Err(SwapperError::InputAmountError { min_amount: Some("1".into()) });
        }

        if let Some(route) = self.route_cache.get_route(&from_token, &to_token)
            && let Some(simulation) = self.try_quote_candidates(&from_token, &to_token, route, &amount, slippage_bps, require_v2).await?
        {
            return Ok(simulation);
        }

        if let Some(simulation) = self
            .try_quote_candidates(&from_token, &to_token, static_candidates(&from_token, &to_token), &amount, slippage_bps, require_v2)
            .await?
        {
            return Ok(simulation);
        }

        let (cached_candidates, _) = self.route_cache.get(&from_token, &to_token);
        if let Some(simulation) = self
            .try_quote_candidates(&from_token, &to_token, cached_candidates, &amount, slippage_bps, require_v2)
            .await?
        {
            return Ok(simulation);
        }

        if !allow_discovery {
            return Err(SwapperError::NoQuoteAvailable);
        }

        self.discover_and_quote(&from_token, &to_token, &amount, slippage_bps, require_v2).await
    }

    async fn discover_and_quote(&self, from_token: &str, to_token: &str, amount: &BigUint, slippage_bps: u32, require_v2: bool) -> Result<SwapSimulation, SwapperError> {
        let (_, explored) = self.route_cache.get(from_token, to_token);
        let routers = FALLBACK_ROUTERS
            .iter()
            .filter(|router| !require_v2 || router.is_supported_v2())
            .filter(|router| !explored.iter().any(|address| address == router.address))
            .collect::<Vec<_>>();
        let explored_addresses = routers.iter().map(|router| router.address.to_string()).collect::<Vec<_>>();
        let discoveries = join_all(routers.iter().map(|router| self.discover_candidate(from_token, to_token, router))).await;
        let mut candidates = Vec::new();
        let mut error = SwapperError::NoQuoteAvailable;

        for discovery in discoveries {
            let candidate = match discovery {
                Ok(candidate) => candidate,
                Err(err) if is_retryable_get_method_error(&err) => return Err(err),
                Err(err) => {
                    error = err;
                    continue;
                }
            };

            candidates.push(candidate);
        }

        if candidates.is_empty() {
            self.route_cache.put(from_token, to_token, &[], &explored_addresses);
            return Err(error);
        }

        match self.quote_best_candidate(candidates.clone(), from_token, to_token, amount, slippage_bps).await {
            Ok((pool, simulation)) => {
                self.route_cache.put(from_token, to_token, &candidates, &explored_addresses);
                self.route_cache.put_route(from_token, to_token, std::slice::from_ref(&pool));
                Ok(simulation)
            }
            Err(err) if is_retryable_get_method_error(&err) => Err(err),
            Err(err) => {
                self.route_cache.put(from_token, to_token, &candidates, &explored_addresses);
                Err(err)
            }
        }
    }

    async fn discover_candidate(&self, from_token: &str, to_token: &str, router: &RouterInfo) -> Result<DiscoveredPool, SwapperError> {
        let (wallet0, wallet1) = futures::try_join!(self.client.router_jetton_wallet(router, from_token), self.client.router_jetton_wallet(router, to_token))?;
        let pool_address = self.client.get_pool_address(router, &wallet0, &wallet1).await?;
        Ok(DiscoveredPool {
            pool_address,
            router: router_model(router),
            asset0: from_token.to_string(),
            asset1: to_token.to_string(),
            wallet0,
            wallet1,
            lp_fee_bps: None,
        })
    }

    async fn try_quote_candidates(
        &self,
        from_token: &str,
        to_token: &str,
        candidates: Vec<DiscoveredPool>,
        amount: &BigUint,
        slippage_bps: u32,
        require_v2: bool,
    ) -> Result<Option<SwapSimulation>, SwapperError> {
        let candidates = filter_candidates(candidates, require_v2);
        if candidates.is_empty() {
            return Ok(None);
        }
        match self.quote_best_candidate(candidates, from_token, to_token, amount, slippage_bps).await {
            Ok((pool, simulation)) => {
                self.route_cache.put_route(from_token, to_token, std::slice::from_ref(&pool));
                Ok(Some(simulation))
            }
            Err(err) if is_retryable_get_method_error(&err) => Err(err),
            Err(_) => Ok(None),
        }
    }

    async fn quote_best_candidate(
        &self,
        candidates: Vec<DiscoveredPool>,
        from_token: &str,
        to_token: &str,
        amount: &BigUint,
        slippage_bps: u32,
    ) -> Result<(DiscoveredPool, SwapSimulation), SwapperError> {
        let quotes = join_all(
            candidates
                .into_iter()
                .map(|candidate| self.quote_candidate(candidate, from_token, to_token, amount, slippage_bps)),
        )
        .await;
        let mut best_quote: Option<(DiscoveredPool, SwapSimulation)> = None;
        for quote in quotes {
            let quote = match quote {
                Ok(quote) => quote,
                Err(err) if is_retryable_get_method_error(&err) => return Err(err),
                Err(_) => continue,
            };
            let quote_amount = BigUint::from_str(&quote.1.ask_units)?;
            let is_best = match &best_quote {
                Some((_, best)) => quote_amount > BigUint::from_str(&best.ask_units)?,
                None => true,
            };
            if is_best {
                best_quote = Some(quote);
            }
        }
        best_quote.ok_or(SwapperError::NoQuoteAvailable)
    }

    async fn quote_candidate(
        &self,
        candidate: DiscoveredPool,
        from_token: &str,
        to_token: &str,
        amount: &BigUint,
        slippage_bps: u32,
    ) -> Result<(DiscoveredPool, SwapSimulation), SwapperError> {
        let pool_data = self.client.get_pool_data(&candidate.pool_address).await?;
        if pool_data.is_locked {
            return Err(SwapperError::NoQuoteAvailable);
        }
        if let Some(lp_fee_bps) = candidate.lp_fee_bps
            && pool_data.lp_fee != lp_fee_bps
        {
            return Err(SwapperError::NoQuoteAvailable);
        }
        let offer_wallet = candidate.wallet_for(from_token).ok_or(SwapperError::InvalidRoute)?;
        let ask_wallet = candidate.wallet_for(to_token).ok_or(SwapperError::InvalidRoute)?;
        let ask_units = compute_amount_out(&pool_data, offer_wallet, amount)?;
        if ask_units == BigUint::from(0u8) {
            return Err(SwapperError::NoQuoteAvailable);
        }
        let min_ask_units = apply_slippage(&ask_units, slippage_bps);
        let simulation = SwapSimulation {
            offer_jetton_wallet: offer_wallet.to_string(),
            ask_jetton_wallet: ask_wallet.to_string(),
            router: candidate.router.clone(),
            ask_units: ask_units.to_string(),
            min_ask_units: min_ask_units.to_string(),
        };
        Ok((candidate, simulation))
    }

    async fn get_quotes(&self, request: &QuoteRequest, from_value: &str) -> Result<QuotePath, SwapperError> {
        if request.from_asset.is_native() || request.to_asset.is_native() {
            return self.quote_direct(request, from_value, true).await;
        }

        let direct = self.quote_direct(request, from_value, false).await;
        let intermediary_paths = self.quote_intermediary_paths(request, from_value, false).await;
        if let Some(err) = retryable_path_error(std::iter::once(&direct).chain(intermediary_paths.iter())) {
            return Err(err);
        }
        if let Ok(path) = Self::select_best_quote_path(std::iter::once(direct).chain(intermediary_paths)) {
            return Ok(path);
        }

        let intermediary_paths = self.quote_intermediary_paths(request, from_value, true).await;
        if let Some(err) = retryable_path_error(intermediary_paths.iter()) {
            return Err(err);
        }
        if let Ok(path) = Self::select_best_quote_path(intermediary_paths) {
            return Ok(path);
        }

        self.quote_direct(request, from_value, true).await
    }

    fn select_best_quote_path(paths: impl IntoIterator<Item = Result<QuotePath, SwapperError>>) -> Result<QuotePath, SwapperError> {
        let mut error = None;
        let mut best = None;
        for result in paths {
            match result {
                Ok(path) => {
                    let amount = BigUint::from_str(&path.to_value)?;
                    let is_best = match &best {
                        Some((best_amount, _)) => amount > *best_amount,
                        None => true,
                    };
                    if is_best {
                        best = Some((amount, path));
                    }
                }
                Err(err) => {
                    if error.is_none() {
                        error = Some(err);
                    }
                }
            }
        }
        match best {
            Some((_, path)) => Ok(path),
            None => match error {
                Some(error) => Err(error),
                None => Err(SwapperError::NoQuoteAvailable),
            },
        }
    }
}

#[async_trait]
impl<C> Swapper for Stonfi<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        vec![SwapperChainAsset::All(Chain::Ton)]
    }

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let from_value = quote_value_after_reserve_by_chain(request)?;
        let path = self.get_quotes(request, &from_value).await?;

        Ok(Quote {
            from_value,
            to_value: path.to_value,
            data: ProviderData {
                provider: self.provider().clone(),
                routes: path.routes,
                slippage_bps: request.options.slippage.bps,
            },
            request: request.clone(),
            eta_in_seconds: None,
        })
    }

    async fn get_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let simulations = quote
            .data
            .routes
            .iter()
            .map(|route| serde_json::from_str::<SwapSimulation>(&route.route_data).map_err(|_| SwapperError::InvalidRoute))
            .collect::<Result<Vec<_>, _>>()?;
        if simulations.len() > 2 {
            return Err(SwapperError::InvalidRoute);
        }
        let simulation = simulations.first().ok_or(SwapperError::InvalidRoute)?;
        let next_swap = simulations
            .get(1)
            .map(|next_simulation| {
                Ok::<_, SwapperError>(NextSwapParams {
                    simulation: next_simulation,
                    min_ask_amount: scaled_next_min_ask_amount(simulation, next_simulation)?,
                })
            })
            .transpose()?;
        let referral_fee = Self::referral_fee();
        let receiver_address = if quote.request.destination_address.is_empty() {
            &quote.request.wallet_address
        } else {
            &quote.request.destination_address
        };
        let sender_jetton_wallet = self.client.sender_jetton_wallet(quote).await?;

        let tx = build_swap_transaction(SwapTransactionParams {
            simulation,
            next_swap,
            from_native: quote.request.from_asset.is_native(),
            to_native: quote.request.to_asset.is_native(),
            sender_jetton_wallet: sender_jetton_wallet.as_deref(),
            from_value: &quote.from_value,
            wallet_address: Address::parse(&quote.request.wallet_address)?,
            receiver_address: Address::parse(receiver_address)?,
            referral: ReferralParams {
                address: Address::parse(&referral_fee.address)?,
                bps: referral_fee.bps,
            },
            deadline: None,
        })?;

        Ok(SwapperQuoteData::new_contract(tx.to, tx.value, tx.data, None, None))
    }
}

fn is_retryable_get_method_error(err: &SwapperError) -> bool {
    match err {
        SwapperError::ComputeQuoteError(message) => {
            let message = message.to_ascii_lowercase();
            message.contains("ratelimit") || message.contains("rate limit") || message.contains("429") || message.contains("too many requests")
        }
        _ => false,
    }
}

fn retryable_path_error<'a>(paths: impl IntoIterator<Item = &'a Result<QuotePath, SwapperError>>) -> Option<SwapperError> {
    paths.into_iter().find_map(|path| match path {
        Err(err) if is_retryable_get_method_error(err) => Some(err.clone()),
        Ok(_) | Err(_) => None,
    })
}

fn filter_candidates(candidates: Vec<DiscoveredPool>, require_v2: bool) -> Vec<DiscoveredPool> {
    if require_v2 {
        candidates.into_iter().filter(|candidate| candidate_has_supported_version(candidate, true)).collect()
    } else {
        candidates
    }
}

fn route_has_supported_version(route: &[DiscoveredPool], require_v2: bool) -> bool {
    !route.is_empty() && route.iter().all(|candidate| candidate_has_supported_version(candidate, require_v2))
}

fn candidate_has_supported_version(candidate: &DiscoveredPool, require_v2: bool) -> bool {
    !require_v2 || candidate.router.is_supported_v2()
}

#[cfg(test)]
mod tests {
    use super::super::constants::STATIC_POOLS;
    use super::*;
    use crate::Options;
    use gem_ton::constants::TON_PROXY_JETTON_ADDRESS;
    use primitives::{asset_constants::TON_USDT_TOKEN_ID, testkit::signer_mock::TEST_TON_SENDER};
    use std::sync::{Arc, Mutex};

    const PTON_WALLET: &str = "EQCSIMGBps_qzRG3uPYhON8bucyCtu0mYdL1-u4gSz77IBa3";
    const USDT_WALLET: &str = "EQCSLWJ9fY7b0A5OI72wxUp27l4fRlc6GvRBeFf6PiPpH4p3";
    const GRAM_TOKEN_ID: &str = "EQC47093oX5Xhb0xuk2lCr2RhS8rj-vul61u4W2UH5ORmG_O";
    const DISCOVERED_POOL: &str = "EQCGScrZe1xbyWqWDvdI6mzP-GAcAWFv6ZXuaJOuSqemxku4";
    const V1_POOL: &str = "EQD8TJ8xEWB1SpnRE4d89YO3jl0W0EiBnNS4IBaHaUmdfizE";

    fn discovered_pool(pool_address: &str) -> DiscoveredPool {
        DiscoveredPool {
            pool_address: pool_address.to_string(),
            router: router_model(&FALLBACK_ROUTERS[0]),
            asset0: TON_PROXY_JETTON_ADDRESS.to_string(),
            asset1: TON_USDT_TOKEN_ID.to_string(),
            wallet0: PTON_WALLET.to_string(),
            wallet1: USDT_WALLET.to_string(),
            lp_fee_bps: None,
        }
    }

    fn get_method_response(stack: serde_json::Value) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "ok": true,
            "result": {
                "exit_code": 0,
                "stack": stack
            }
        }))
        .unwrap()
    }

    fn cell_response(address: &str) -> Vec<u8> {
        let bytes = Address::parse(address).unwrap().to_boc_base64().unwrap();
        get_method_response(serde_json::json!([["cell", { "bytes": bytes }]]))
    }

    fn get_pool_data_response(is_locked: bool, reserve0: u64, reserve1: u64, token0_wallet: &str, token1_wallet: &str, lp_fee_bps: u32) -> Vec<u8> {
        let token0 = Address::parse(token0_wallet).unwrap().to_boc_base64().unwrap();
        let token1 = Address::parse(token1_wallet).unwrap().to_boc_base64().unwrap();
        get_method_response(serde_json::json!([
            ["num", if is_locked { "0x1" } else { "0x0" }],
            ["num", "0x0"],
            ["num", "0x0"],
            ["num", reserve0.to_string()],
            ["num", reserve1.to_string()],
            ["cell", { "bytes": token0 }],
            ["cell", { "bytes": token1 }],
            ["num", lp_fee_bps.to_string()],
            ["num", "0x3"],
            ["num", "0x0"],
            ["num", "0x0"],
            ["cell", { "bytes": token1 }]
        ]))
    }

    fn get_v1_pool_data_response(reserve0: u64, reserve1: u64, token0_wallet: &str, token1_wallet: &str, lp_fee_bps: u32) -> Vec<u8> {
        let token0 = Address::parse(token0_wallet).unwrap().to_boc_base64().unwrap();
        let token1 = Address::parse(token1_wallet).unwrap().to_boc_base64().unwrap();
        get_method_response(serde_json::json!([
            ["num", reserve0.to_string()],
            ["num", reserve1.to_string()],
            ["cell", { "bytes": token0 }],
            ["cell", { "bytes": token1 }],
            ["num", lp_fee_bps.to_string()],
            ["num", "0xa"],
            ["num", "0xa"],
            ["cell", { "bytes": token1 }],
            ["num", "0x0"],
            ["num", "0x0"]
        ]))
    }

    fn not_ton_pool() -> &'static super::super::constants::StaticPool {
        STATIC_POOLS.iter().find(|pool| pool.token0.ends_with("__NOT")).unwrap()
    }

    fn v1_ton_usdt_pool() -> &'static super::super::constants::StaticPool {
        STATIC_POOLS.iter().find(|pool| pool.pool_address == V1_POOL).unwrap()
    }

    fn provider_with_get_method<F>(handler: F) -> Stonfi<gem_client::testkit::MockClient>
    where
        F: Fn(&str, &str) -> Vec<u8> + Send + Sync + 'static,
    {
        Stonfi::new_with_client(TonClient::new(gem_client::testkit::MockClient::new().with_post(move |_, body| {
            let request: serde_json::Value = serde_json::from_slice(body).unwrap();
            let address = request["address"].as_str().unwrap();
            let method = request["method"].as_str().unwrap();
            Ok(handler(method, address))
        })))
    }

    fn provider_with_pool_data<F>(handler: F) -> Stonfi<gem_client::testkit::MockClient>
    where
        F: Fn(&str) -> Vec<u8> + Send + Sync + 'static,
    {
        provider_with_get_method(move |_, address| handler(address))
    }

    #[tokio::test]
    async fn test_intermediary_discovery_minimizes_calls() {
        let not_pool = not_ton_pool();
        let calls = Arc::new(Mutex::new(Vec::<String>::new()));
        let calls_ref = calls.clone();
        let provider = provider_with_get_method(move |method, address| {
            calls_ref.lock().unwrap().push(method.to_string());
            match method {
                "get_wallet_address" => cell_response(USDT_WALLET),
                "get_pool_address" => cell_response(DISCOVERED_POOL),
                "get_pool_data" if address == DISCOVERED_POOL => get_pool_data_response(false, 3_000_000_000_000, 1_800_000_000_000_000, USDT_WALLET, PTON_WALLET, 7),
                "get_pool_data" if address == not_pool.pool_address => {
                    get_pool_data_response(false, 5_000_000_000_000_000, 2_000_000_000_000, not_pool.token0_wallet, not_pool.token1_wallet, 20)
                }
                _ => unreachable!("{method} {address}"),
            }
        });
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_token(Chain::Ton, "unknown-token")),
            to_asset: SwapperQuoteAsset::from(AssetId::from_token(Chain::Ton, not_pool.token0)),
            wallet_address: TEST_TON_SENDER.to_string(),
            destination_address: TEST_TON_SENDER.to_string(),
            value: "1000000000".to_string(),
            options: Options::new_with_slippage(100.into()),
        };

        let quote = provider.get_quote(&request).await.unwrap();
        assert_eq!(quote.data.routes.len(), 2);
        assert_eq!(
            calls.lock().unwrap().as_slice(),
            ["get_wallet_address", "get_pool_address", "get_pool_data", "get_pool_data"]
        );

        calls.lock().unwrap().clear();
        let quote = provider.get_quote(&request).await.unwrap();
        assert_eq!(quote.data.routes.len(), 2);
        assert_eq!(calls.lock().unwrap().as_slice(), ["get_pool_data", "get_pool_data"]);
    }

    #[tokio::test]
    async fn test_quote_candidate_rejects_locked_pool() {
        let provider = provider_with_pool_data(|_| get_pool_data_response(true, 3_809_436_784_065, 1_784_561_670_122_756, USDT_WALLET, PTON_WALLET, 7));
        let amount = BigUint::from(1_000_000_000u64);

        assert_eq!(
            provider
                .quote_candidate(discovered_pool("pool-a"), TON_PROXY_JETTON_ADDRESS, TON_USDT_TOKEN_ID, &amount, 100)
                .await
                .unwrap_err(),
            SwapperError::NoQuoteAvailable
        );
    }

    #[tokio::test]
    async fn test_direct_quote_can_select_v1_static_pool() {
        let v1_pool = v1_ton_usdt_pool();
        let provider = provider_with_pool_data(move |address| match address {
            DISCOVERED_POOL => get_pool_data_response(false, 1_000_000_000_000, 1, PTON_WALLET, USDT_WALLET, 7),
            V1_POOL => get_v1_pool_data_response(1_000_000_000, 10_000_000_000, v1_pool.token0_wallet, v1_pool.token1_wallet, 20),
            _ => unreachable!("{address}"),
        });
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Ton)),
            to_asset: SwapperQuoteAsset::from(AssetId::from_token(Chain::Ton, TON_USDT_TOKEN_ID)),
            wallet_address: TEST_TON_SENDER.to_string(),
            destination_address: TEST_TON_SENDER.to_string(),
            value: "100000000".to_string(),
            options: Options::new_with_slippage(100.into()),
        };

        let path = provider.quote_direct(&request, &request.value, false).await.unwrap();
        let simulation: SwapSimulation = serde_json::from_str(&path.routes[0].route_data).unwrap();

        assert_eq!(path.routes.len(), 1);
        assert_eq!(simulation.router.major_version, 1);
        assert_eq!(simulation.offer_jetton_wallet, v1_pool.token0_wallet);
        assert_eq!(simulation.ask_jetton_wallet, v1_pool.token1_wallet);
    }

    #[tokio::test]
    async fn test_discovered_direct_quote_selects_best_router_pool() {
        let calls = Arc::new(Mutex::new(Vec::<String>::new()));
        let calls_ref = calls.clone();
        let provider = provider_with_get_method(move |method, address| {
            calls_ref.lock().unwrap().push(format!("{method} {address}"));
            match method {
                "get_wallet_address" if address == TON_USDT_TOKEN_ID => cell_response(USDT_WALLET),
                "get_wallet_address" if address == GRAM_TOKEN_ID => cell_response(PTON_WALLET),
                "get_pool_address" if address == FALLBACK_ROUTERS[0].address => cell_response(DISCOVERED_POOL),
                "get_pool_address" if address == FALLBACK_ROUTERS[1].address => cell_response(V1_POOL),
                "get_pool_data" if address == DISCOVERED_POOL => get_pool_data_response(false, 1, 19_811_277, USDT_WALLET, PTON_WALLET, 20),
                "get_pool_data" if address == V1_POOL => get_v1_pool_data_response(226_348_366, 194_933_327_038_860, USDT_WALLET, PTON_WALLET, 20),
                _ => unreachable!("{method} {address}"),
            }
        });
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_token(Chain::Ton, TON_USDT_TOKEN_ID)),
            to_asset: SwapperQuoteAsset::from(AssetId::from_token(Chain::Ton, GRAM_TOKEN_ID)),
            wallet_address: TEST_TON_SENDER.to_string(),
            destination_address: TEST_TON_SENDER.to_string(),
            value: "233000".to_string(),
            options: Options::new_with_slippage(100.into()),
        };

        let path = provider.quote_direct(&request, &request.value, true).await.unwrap();
        let simulation: SwapSimulation = serde_json::from_str(&path.routes[0].route_data).unwrap();
        let mut pool_calls = calls.lock().unwrap().iter().filter(|call| call.starts_with("get_pool_data ")).cloned().collect::<Vec<_>>();
        pool_calls.sort();
        let mut expected_pool_calls = vec![format!("get_pool_data {DISCOVERED_POOL}"), format!("get_pool_data {V1_POOL}")];
        expected_pool_calls.sort();

        assert_eq!(pool_calls, expected_pool_calls);
        assert_eq!(simulation.router.major_version, 1);
        assert_eq!(simulation.ask_units, "199854680472");
    }

    #[tokio::test]
    async fn test_quote_best_candidate_selects_largest_output() {
        let provider = provider_with_pool_data(|address| match address {
            "pool-a" => get_pool_data_response(false, 3_000_000_000_000, 1_800_000_000_000_000, USDT_WALLET, PTON_WALLET, 7),
            "pool-b" => get_pool_data_response(false, 4_000_000_000_000, 1_800_000_000_000_000, USDT_WALLET, PTON_WALLET, 7),
            _ => unreachable!(),
        });
        let amount = BigUint::from(1_000_000_000u64);
        let (pool, simulation) = provider
            .quote_best_candidate(
                vec![discovered_pool("pool-a"), discovered_pool("pool-b")],
                TON_PROXY_JETTON_ADDRESS,
                TON_USDT_TOKEN_ID,
                &amount,
                100,
            )
            .await
            .unwrap();

        assert_eq!(pool.pool_address, "pool-b");
        assert_eq!(simulation.ask_units, "2219998");
    }

    #[test]
    fn test_intermediary_discovery_requires_known_leg() {
        let provider = provider_with_pool_data(|_| unreachable!());
        let unknown_a = SwapperQuoteAsset::from(AssetId::from_token(Chain::Ton, "unknown-a"));
        let unknown_b = SwapperQuoteAsset::from(AssetId::from_token(Chain::Ton, "unknown-b"));
        let ton = SwapperQuoteAsset::from(AssetId::from_chain(Chain::Ton));
        let usdt = SwapperQuoteAsset::from(TON_USDT_ASSET_ID.clone());

        assert!(!provider.should_quote_intermediary_path(&unknown_a, &ton, &unknown_b, true));
        assert!(provider.should_quote_intermediary_path(&unknown_a, &ton, &usdt, true));
        assert!(!provider.should_quote_intermediary_path(&unknown_a, &ton, &usdt, false));
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::{Options, SwapperQuoteAsset, alien::reqwest_provider::NativeProvider, stonfi::testkit::NOT_TOKEN_ID, testkit::mock_ton};
    use primitives::{AssetId, asset_constants::TON_USDT_ASSET_ID, testkit::signer_mock::TEST_TON_SENDER};

    #[tokio::test]
    async fn test_stonfi_quote_and_quote_data_ton_to_usdt() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::default());
        let provider = Stonfi::new(rpc_provider);
        let request = mock_ton(TEST_TON_SENDER.to_string());

        let quote = provider.get_quote(&request).await?;
        assert_eq!(quote.from_value, request.value);
        assert!(quote.to_value.parse::<u64>().unwrap() > 0);
        assert_eq!(quote.data.provider, provider.provider().clone());
        assert_eq!(quote.data.routes.len(), 1);
        println!("STON.fi TON -> USDT quote: {quote:?}");

        let quote_data = provider.get_quote_data(&quote, FetchQuoteData::None).await?;
        assert!(!quote_data.to.is_empty());
        assert!(!quote_data.value.is_empty());
        assert!(quote_data.data.starts_with("te6cc"));
        println!("STON.fi TON -> USDT quote_data: {quote_data:?}");

        Ok(())
    }

    #[tokio::test]
    async fn test_stonfi_quote_and_quote_data_not_to_usdt() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::default());
        let provider = Stonfi::new(rpc_provider);
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_token(Chain::Ton, NOT_TOKEN_ID)),
            to_asset: SwapperQuoteAsset::from(TON_USDT_ASSET_ID.clone()),
            wallet_address: TEST_TON_SENDER.to_string(),
            destination_address: TEST_TON_SENDER.to_string(),
            value: "1000000000000".to_string(),
            options: Options::new_with_slippage(100.into()),
        };

        let quote = provider.get_quote(&request).await?;
        assert_eq!(quote.from_value, request.value);
        assert!(quote.to_value.parse::<u64>().unwrap() > 0);
        assert_eq!(quote.data.provider, provider.provider().clone());
        assert_eq!(quote.data.routes.len(), 2);
        assert_eq!(quote.data.routes[0].input, AssetId::from_token(Chain::Ton, NOT_TOKEN_ID));
        assert_eq!(quote.data.routes[0].output, AssetId::from_chain(Chain::Ton));
        assert_eq!(quote.data.routes[1].input, AssetId::from_chain(Chain::Ton));
        assert_eq!(quote.data.routes[1].output, TON_USDT_ASSET_ID.clone());
        println!("STON.fi NOT -> USDT quote: {quote:?}");

        let quote_data = provider.get_quote_data(&quote, FetchQuoteData::None).await?;
        assert!(!quote_data.to.is_empty());
        assert!(!quote_data.value.is_empty());
        assert!(quote_data.data.starts_with("te6cc"));
        println!("STON.fi NOT -> USDT quote_data: {quote_data:?}");

        Ok(())
    }
}
