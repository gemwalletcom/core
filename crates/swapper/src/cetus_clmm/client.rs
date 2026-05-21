use super::{
    cache::PoolCache,
    constants::{CETUS_ALL_TICK_SPACINGS, CETUS_PRIMARY_TICK_SPACINGS, KNOWN_POOLS},
    model::{DiscoveredPool, Hop, INTERMEDIATE_COIN_TYPES},
    tx_builder,
};
use crate::{
    ProviderType, RpcProvider, SwapperError, SwapperProvider,
    client_factory::create_sui_client,
    fees::{ReferralFee, default_referral_fees},
};
use gem_sui::{EMPTY_ADDRESS, SUI_COIN_TYPE, SuiClient, coin_type_matches, full_coin_type, models::InspectResult, tx_builder::ObjectResolver};
use primitives::AssetId;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

#[derive(Debug, Clone)]
pub(super) struct QuoteResult {
    pub amount_out: u64,
    pub current_sqrt_price: u128,
    pub after_sqrt_price: u128,
    pub is_exceed: bool,
}

const DIRECT_PRICE_IMPACT_THRESHOLD_BPS: u32 = 50;

#[derive(Debug, Default)]
struct PhaseResult {
    acceptable_direct: Option<(Vec<Hop>, u32)>,
    best_route: Option<(Vec<Hop>, u32)>,
}

pub struct CetusClmm {
    pub(super) provider: ProviderType,
    pub(super) sui_client: SuiClient,
    pool_cache: PoolCache,
}

impl std::fmt::Debug for CetusClmm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CetusClmm")
    }
}

impl CetusClmm {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        let sui_client = create_sui_client(rpc_provider).expect("failed to create Sui gRPC client");
        Self::with_client(sui_client)
    }

    pub fn with_client(sui_client: SuiClient) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::CetusClmm),
            sui_client,
            pool_cache: PoolCache::default(),
        }
    }

    pub(super) fn referral_fee() -> ReferralFee {
        default_referral_fees().sui
    }

    pub(super) fn coin_type(asset_id: &AssetId) -> String {
        full_coin_type(asset_id.token_id.as_deref().unwrap_or(SUI_COIN_TYPE))
    }

    pub(super) async fn find_route_hops(&self, from: &str, to: &str, swap_amount: u64) -> Result<Vec<Hop>, SwapperError> {
        if let Some(cached_route) = self.pool_cache.get_route(from, to) {
            let quotes = self.quote_candidates_batched(vec![cached_route], from, swap_amount).await;
            if let Some((hops, _)) = quotes.into_iter().flatten().next() {
                return Ok(hops);
            }
        }

        let primary = self.try_route_with_ticks(from, to, swap_amount, CETUS_PRIMARY_TICK_SPACINGS).await;
        if let Some((hops, _)) = primary.acceptable_direct {
            self.cache_winning_route(from, to, &hops);
            return Ok(hops);
        }
        if let Some((hops, impact)) = &primary.best_route
            && *impact <= DIRECT_PRICE_IMPACT_THRESHOLD_BPS
        {
            self.cache_winning_route(from, to, hops);
            return Ok(hops.clone());
        }

        let expanded = self.try_route_with_ticks(from, to, swap_amount, CETUS_ALL_TICK_SPACINGS).await;
        let (hops, _) = expanded
            .acceptable_direct
            .or(expanded.best_route)
            .or(primary.best_route)
            .ok_or(SwapperError::NoQuoteAvailable)?;
        self.cache_winning_route(from, to, &hops);
        Ok(hops)
    }

    async fn try_route_with_ticks(&self, from: &str, to: &str, swap_amount: u64, ticks: &[u32]) -> PhaseResult {
        let direct_candidates: Vec<Vec<DiscoveredPool>> = self.discover_direct_pools(from, to, ticks).await.into_iter().map(|pool| vec![pool]).collect();
        let direct_quotes = self.quote_candidates_batched(direct_candidates, from, swap_amount).await;
        let acceptable_direct = direct_quotes
            .iter()
            .filter_map(|q| q.as_ref())
            .filter(|(_, impact)| *impact < DIRECT_PRICE_IMPACT_THRESHOLD_BPS)
            .max_by_key(|(hops, _)| hops.last().map(|h| h.amount_out).unwrap_or_default())
            .cloned();
        if acceptable_direct.is_some() {
            return PhaseResult {
                acceptable_direct,
                best_route: None,
            };
        }

        let mut multi_hop_candidates: Vec<Vec<DiscoveredPool>> = Vec::new();
        for raw_intermediate in INTERMEDIATE_COIN_TYPES {
            let intermediate = full_coin_type(raw_intermediate);
            if coin_type_matches(from, &intermediate) || coin_type_matches(to, &intermediate) {
                continue;
            }
            let (firsts, seconds) = futures::future::join(self.discover_direct_pools(from, &intermediate, ticks), self.discover_direct_pools(&intermediate, to, ticks)).await;
            for first in &firsts {
                for second in &seconds {
                    multi_hop_candidates.push(vec![first.clone(), second.clone()]);
                }
            }
        }
        let multi_hop_quotes = self.quote_candidates_batched(multi_hop_candidates, from, swap_amount).await;
        let best_route = direct_quotes
            .into_iter()
            .chain(multi_hop_quotes)
            .flatten()
            .max_by_key(|(hops, _)| hops.last().map(|h| h.amount_out).unwrap_or_default());
        PhaseResult {
            acceptable_direct: None,
            best_route,
        }
    }

    fn cache_winning_route(&self, from: &str, to: &str, hops: &[Hop]) {
        let route: Vec<DiscoveredPool> = hops
            .iter()
            .map(|hop| DiscoveredPool {
                pool_id: hop.pool_id.clone(),
                pool_init_version: hop.pool_init_version,
                coin_a: hop.coin_a.clone(),
                coin_b: hop.coin_b.clone(),
            })
            .collect();
        self.pool_cache.put_route(from, to, &route);
    }

    fn known_pools(from: &str, to: &str) -> Vec<DiscoveredPool> {
        KNOWN_POOLS
            .iter()
            .filter(|known| {
                (coin_type_matches(from, known.coin_a) && coin_type_matches(to, known.coin_b)) || (coin_type_matches(from, known.coin_b) && coin_type_matches(to, known.coin_a))
            })
            .map(|known| DiscoveredPool {
                pool_id: known.pool_id.to_string(),
                pool_init_version: known.pool_init_version,
                coin_a: known.coin_a.to_string(),
                coin_b: known.coin_b.to_string(),
            })
            .collect()
    }

    async fn discover_direct_pools(&self, from: &str, to: &str, ticks: &[u32]) -> Vec<DiscoveredPool> {
        let known = Self::known_pools(from, to);
        if !known.is_empty() {
            return known;
        }
        let (cached_pools, explored) = self.pool_cache.get(from, to);
        let missing: Vec<u32> = ticks.iter().filter(|t| !explored.contains(t)).copied().collect();
        if missing.is_empty() {
            return cached_pools;
        }
        let Some(new_pools) = self.query_direct_pools(from, to, &missing).await else {
            return cached_pools;
        };
        self.pool_cache.put(from, to, &new_pools, &missing);
        self.pool_cache.get(from, to).0
    }

    async fn query_direct_pools(&self, from: &str, to: &str, ticks: &[u32]) -> Option<Vec<DiscoveredPool>> {
        let (coin_a, coin_b) = canonical_pair_order(from, to);
        let inspects = ticks.iter().map(|tick| self.inspect_pool_id(coin_a, coin_b, *tick));
        let results = futures::future::join_all(inspects).await;
        let mut candidates: Vec<(String, String, String)> = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();
        for result in results {
            match result {
                Ok(Some(pool_id)) => {
                    if seen.insert(pool_id.clone()) {
                        candidates.push((pool_id, coin_a.to_string(), coin_b.to_string()));
                    }
                }
                Ok(None) => {}
                Err(_) => return None,
            }
        }
        if candidates.is_empty() {
            return Some(Vec::new());
        }
        let pool_ids: Vec<String> = candidates.iter().map(|(id, _, _)| id.clone()).collect();
        let resolver = ObjectResolver::prefetch(&self.sui_client, pool_ids, &HashMap::new()).await.ok()?;
        Some(
            candidates
                .into_iter()
                .filter_map(|(pool_id, coin_a, coin_b)| {
                    let pool_init_version = resolver.initial_shared_version(&pool_id)?;
                    Some(DiscoveredPool {
                        pool_id,
                        pool_init_version,
                        coin_a,
                        coin_b,
                    })
                })
                .collect(),
        )
    }

    async fn quote_candidates_batched(&self, candidates: Vec<Vec<DiscoveredPool>>, from: &str, swap_amount: u64) -> Vec<Option<(Vec<Hop>, u32)>> {
        if candidates.is_empty() {
            return Vec::new();
        }
        if candidates[0].len() == 1 {
            self.quote_direct_batched(candidates, from, swap_amount).await
        } else {
            self.quote_multi_hop_fused(candidates, from, swap_amount).await
        }
    }

    async fn quote_direct_batched(&self, candidates: Vec<Vec<DiscoveredPool>>, from: &str, swap_amount: u64) -> Vec<Option<(Vec<Hop>, u32)>> {
        let hops: Vec<Hop> = candidates.iter().map(|pools| pools[0].clone().into_hop(from, swap_amount)).collect();
        let inputs: Vec<(&Hop, u64)> = hops.iter().map(|hop| (hop, swap_amount)).collect();
        let quote_results = self.inspect_batch_quotes(&inputs).await.unwrap_or_else(|_| vec![None; candidates.len()]);

        hops.into_iter()
            .zip(quote_results)
            .map(|(mut hop, quote)| {
                let q = quote?;
                if q.amount_out == 0 || q.is_exceed {
                    return None;
                }
                hop.amount_out = q.amount_out;
                hop.after_sqrt_price = q.after_sqrt_price;
                let impact = price_impact_bps(q.current_sqrt_price, q.after_sqrt_price);
                Some((vec![hop], impact))
            })
            .collect()
    }

    async fn quote_multi_hop_fused(&self, candidates: Vec<Vec<DiscoveredPool>>, from: &str, swap_amount: u64) -> Vec<Option<(Vec<Hop>, u32)>> {
        let hop_pairs: Vec<(Hop, Hop)> = candidates
            .iter()
            .map(|pools| {
                let hop1 = pools[0].clone().into_hop(from, swap_amount);
                let intermediate = hop1.output_coin_type().to_string();
                let hop2 = pools[1].clone().into_hop(&intermediate, 0);
                (hop1, hop2)
            })
            .collect();
        let inputs: Vec<(&Hop, &Hop, u64)> = hop_pairs.iter().map(|(h1, h2)| (h1, h2, swap_amount)).collect();
        let fused_results = self.inspect_batch_multi_hop_quotes(&inputs).await.unwrap_or_else(|_| vec![(None, None); candidates.len()]);

        hop_pairs
            .into_iter()
            .zip(fused_results)
            .map(|((mut hop1, mut hop2), (q1, q2))| {
                let q1 = q1?;
                if q1.amount_out == 0 || q1.is_exceed {
                    return None;
                }
                let q2 = q2?;
                if q2.amount_out == 0 || q2.is_exceed {
                    return None;
                }
                hop1.amount_out = q1.amount_out;
                hop1.after_sqrt_price = q1.after_sqrt_price;
                hop2.amount_in = q1.amount_out;
                hop2.amount_out = q2.amount_out;
                hop2.after_sqrt_price = q2.after_sqrt_price;
                let max_impact = price_impact_bps(q1.current_sqrt_price, q1.after_sqrt_price).max(price_impact_bps(q2.current_sqrt_price, q2.after_sqrt_price));
                Some((vec![hop1, hop2], max_impact))
            })
            .collect()
    }

    async fn inspect_pool_id(&self, coin_a: &str, coin_b: &str, tick_spacing: u32) -> Result<Option<String>, SwapperError> {
        let transaction = tx_builder::build_pool_id_inspect(coin_a, coin_b, tick_spacing)?;
        let Some(result) = self.run_inspect(transaction).await? else {
            return Ok(None);
        };
        let bytes = result
            .results
            .last()
            .and_then(|command| command.return_values.first())
            .map(|(bytes, _)| bytes)
            .ok_or_else(|| SwapperError::ComputeQuoteError("Cetus CLMM pool discovery returned no value".into()))?;
        if bytes.len() != 32 {
            return Err(SwapperError::ComputeQuoteError("Cetus CLMM pool discovery returned invalid id".into()));
        }
        Ok(Some(format!("0x{}", hex::encode(bytes))))
    }

    async fn inspect_batch_quotes(&self, quotes: &[(&Hop, u64)]) -> Result<Vec<Option<QuoteResult>>, SwapperError> {
        if quotes.is_empty() {
            return Ok(Vec::new());
        }
        let result = self
            .run_inspect(tx_builder::build_batch_quote_inspect(quotes)?)
            .await?
            .ok_or(SwapperError::NoQuoteAvailable)?;
        Ok((0..quotes.len()).map(|i| quote_result_at(&result, i)).collect())
    }

    async fn inspect_batch_multi_hop_quotes(&self, routes: &[(&Hop, &Hop, u64)]) -> Result<Vec<(Option<QuoteResult>, Option<QuoteResult>)>, SwapperError> {
        if routes.is_empty() {
            return Ok(Vec::new());
        }
        let result = self
            .run_inspect(tx_builder::build_batch_multi_hop_quote_inspect(routes)?)
            .await?
            .ok_or(SwapperError::NoQuoteAvailable)?;
        Ok((0..routes.len()).map(|i| (quote_result_at(&result, i * 3), quote_result_at(&result, i * 3 + 2))).collect())
    }

    async fn run_inspect(&self, transaction: Vec<u8>) -> Result<Option<InspectResult>, SwapperError> {
        let result = self
            .sui_client
            .inspect_transaction_block(EMPTY_ADDRESS, &transaction, None)
            .await
            .map_err(SwapperError::compute_quote_error)?;
        if result.error.is_some() {
            return Ok(None);
        }
        Ok(Some(result))
    }
}

fn canonical_pair_order<'a>(a: &'a str, b: &'a str) -> (&'a str, &'a str) {
    if a > b { (a, b) } else { (b, a) }
}

fn decode_quote_result_bytes(bytes: &[u8]) -> Result<QuoteResult, SwapperError> {
    if bytes.len() < 66 {
        return Err(SwapperError::ComputeQuoteError("Cetus CLMM quote inspect returned truncated CalculatedSwapResult".into()));
    }
    let amount_out = u64::from_le_bytes(
        bytes[8..16]
            .try_into()
            .map_err(|_| SwapperError::ComputeQuoteError("Cetus CLMM amount_out decode failed".into()))?,
    );
    let after_sqrt_price = u128::from_le_bytes(
        bytes[32..48]
            .try_into()
            .map_err(|_| SwapperError::ComputeQuoteError("Cetus CLMM after_sqrt_price decode failed".into()))?,
    );
    let is_exceed = bytes[48] != 0;
    let current_sqrt_price = u128::from_le_bytes(
        bytes[50..66]
            .try_into()
            .map_err(|_| SwapperError::ComputeQuoteError("Cetus CLMM current_sqrt_price decode failed".into()))?,
    );
    Ok(QuoteResult {
        amount_out,
        current_sqrt_price,
        after_sqrt_price,
        is_exceed,
    })
}

fn price_impact_bps(current_sqrt_price: u128, after_sqrt_price: u128) -> u32 {
    if current_sqrt_price == 0 {
        return u32::MAX;
    }
    let (high, low) = if current_sqrt_price >= after_sqrt_price {
        (current_sqrt_price, after_sqrt_price)
    } else {
        (after_sqrt_price, current_sqrt_price)
    };
    let delta = high - low;
    let bps = delta.saturating_mul(20_000) / current_sqrt_price;
    u32::try_from(bps).unwrap_or(u32::MAX)
}

fn quote_result_at(result: &InspectResult, cmd_idx: usize) -> Option<QuoteResult> {
    let bytes = result.results.get(cmd_idx).and_then(|cmd| cmd.return_values.first()).map(|(bytes, _)| bytes)?;
    decode_quote_result_bytes(bytes).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inspect_result_many(per_command: Vec<Vec<u8>>) -> InspectResult {
        InspectResult {
            effects: gem_sui::models::InspectEffects {
                gas_used: gem_sui::models::InspectGasUsed {
                    computation_cost: 0,
                    storage_cost: 0,
                    storage_rebate: 0,
                },
            },
            events: serde_json::Value::Null,
            error: None,
            results: per_command
                .into_iter()
                .map(|bytes| gem_sui::models::InspectCommandResult {
                    return_values: vec![(bytes, "CalculatedSwapResult".into())],
                })
                .collect(),
        }
    }

    fn calc_swap_bytes(amount_out: u64, current_sqrt: u128, after_sqrt: u128, is_exceed: bool) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(66);
        bytes.extend_from_slice(&997_500_u64.to_le_bytes());
        bytes.extend_from_slice(&amount_out.to_le_bytes());
        bytes.extend_from_slice(&2_500_u64.to_le_bytes());
        bytes.extend_from_slice(&2_500_u64.to_le_bytes());
        bytes.extend_from_slice(&after_sqrt.to_le_bytes());
        bytes.push(if is_exceed { 1 } else { 0 });
        bytes.push(1);
        bytes.extend_from_slice(&current_sqrt.to_le_bytes());
        bytes
    }

    #[test]
    fn test_price_impact_bps() {
        assert_eq!(price_impact_bps(1_000_000, 1_000_000), 0);
        assert_eq!(price_impact_bps(1_000_000, 995_000), 100);
        assert_eq!(price_impact_bps(995_000, 1_000_000), 100);
        assert_eq!(price_impact_bps(1_000_000, 990_000), 200);
        assert_eq!(price_impact_bps(0, 1_000_000), u32::MAX);
    }

    #[test]
    fn test_canonical_pair_order() {
        let sui = gem_sui::SUI_COIN_TYPE_FULL;
        let usdc = "0xdba34672e30cb065b1f93e3ab55318768fd6fef66c15942c9f7cb846e2f900e7::usdc::USDC";
        let blue = "0xe1b45a0e641b9955a20aa0ad1c1f4ad86aad8afb07296d4085e349a50e90bdca::blue::BLUE";

        assert_eq!(canonical_pair_order(usdc, sui), (usdc, sui));
        assert_eq!(canonical_pair_order(sui, usdc), (usdc, sui));
        assert_eq!(canonical_pair_order(blue, sui), (blue, sui));
        assert_eq!(canonical_pair_order(sui, blue), (blue, sui));
        assert_eq!(canonical_pair_order(blue, usdc), (blue, usdc));
        assert_eq!(canonical_pair_order(usdc, blue), (blue, usdc));
    }

    #[test]
    fn test_quote_result_at_extracts_per_command() {
        let current = 521_723_622_374_070_550_528_u128;
        let after = 521_460_761_563_383_315_264_u128;
        let bytes_a = calc_swap_bytes(100_000, current, after, false);
        let bytes_b = calc_swap_bytes(200_000, current, after, true);
        let bytes_c = calc_swap_bytes(300_000, current, after, false);
        let result = inspect_result_many(vec![bytes_a, bytes_b, bytes_c]);

        assert_eq!(quote_result_at(&result, 0).unwrap().amount_out, 100_000);
        assert!(!quote_result_at(&result, 0).unwrap().is_exceed);
        assert_eq!(quote_result_at(&result, 1).unwrap().amount_out, 200_000);
        assert!(quote_result_at(&result, 1).unwrap().is_exceed);
        assert_eq!(quote_result_at(&result, 2).unwrap().amount_out, 300_000);
        assert!(quote_result_at(&result, 3).is_none());
    }

    #[test]
    fn test_decode_quote_result_bytes() {
        let current = 521_723_622_374_070_550_528_u128;
        let after = 521_460_761_563_383_315_264_u128;
        let bytes = calc_swap_bytes(796_985_864, current, after, false);
        let decoded = decode_quote_result_bytes(&bytes).unwrap();
        assert_eq!(decoded.amount_out, 796_985_864);
        assert_eq!(decoded.after_sqrt_price, after);
        assert!(!decoded.is_exceed);

        let exceeded = calc_swap_bytes(796_985_864, current, after, true);
        assert!(decode_quote_result_bytes(&exceeded).unwrap().is_exceed);

        let truncated = decode_quote_result_bytes(&[0u8; 16]);
        match truncated {
            Err(SwapperError::ComputeQuoteError(_)) => {}
            other => panic!("expected ComputeQuoteError, got {other:?}"),
        }
    }
}
