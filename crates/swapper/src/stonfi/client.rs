use super::{constants::RouterInfo, quote::PoolData};
use crate::{Quote, SwapperError, route_cache::ValueCache, static_read_cache_headers};
use gem_client::Client;
use gem_ton::{
    address::Address,
    constants::TON_PROXY_JETTON_ADDRESS,
    models::{RunGetMethodResult, StackArg, StackEntry},
    rpc::client::TonClient,
};
use num_bigint::BigUint;
use num_traits::{Num, ToPrimitive};
use primitives::Address as PrimitiveAddress;
use std::{fmt::Debug, str::FromStr};

const GET_WALLET_ADDRESS_METHOD: &str = "get_wallet_address";
const GET_POOL_ADDRESS_METHOD: &str = "get_pool_address";
const GET_POOL_DATA_METHOD: &str = "get_pool_data";

#[derive(Debug)]
pub(super) struct StonfiClient<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    ton_client: TonClient<C>,
    jetton_wallet_cache: ValueCache<(String, String), String>,
}

impl<C> StonfiClient<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    pub(super) fn new(ton_client: TonClient<C>) -> Self {
        Self {
            ton_client,
            jetton_wallet_cache: ValueCache::default(),
        }
    }

    pub(super) async fn router_jetton_wallet(&self, router: &RouterInfo, token: &str) -> Result<String, SwapperError> {
        if token == TON_PROXY_JETTON_ADDRESS {
            return Ok(router.pton_wallet.to_string());
        }
        self.jetton_wallet(router.address, token).await
    }

    pub(super) async fn get_pool_address(&self, router: &RouterInfo, wallet0: &str, wallet1: &str) -> Result<String, SwapperError> {
        let token0 = Address::parse(wallet0)?;
        let token1 = Address::parse(wallet1)?;
        let result = self
            .run_static_get_method(
                router.address,
                GET_POOL_ADDRESS_METHOD,
                vec![StackArg::slice(token0.to_boc_base64()?), StackArg::slice(token1.to_boc_base64()?)],
            )
            .await?;
        stack_cell_address(&result.stack, 0)
    }

    pub(super) async fn get_pool_data(&self, pool_address: &str) -> Result<PoolData, SwapperError> {
        let result = self.run_get_method(pool_address, GET_POOL_DATA_METHOD, Vec::new()).await?;
        parse_pool_data(&result)
    }

    pub(super) async fn sender_jetton_wallet(&self, quote: &Quote) -> Result<Option<String>, SwapperError> {
        if quote.request.from_asset.is_native() {
            return Ok(None);
        }
        let token_id = quote.request.from_asset.asset_id().token_id.ok_or(SwapperError::NotSupportedAsset)?;
        Ok(Some(self.jetton_wallet(&quote.request.wallet_address, &token_id).await?))
    }

    async fn jetton_wallet(&self, owner: &str, token: &str) -> Result<String, SwapperError> {
        let key = (owner.to_string(), token.to_string());
        if let Some(wallet) = self.jetton_wallet_cache.get(&key) {
            return Ok(wallet);
        }
        let owner_address = Address::parse(owner)?;
        let result = self
            .run_static_get_method(token, GET_WALLET_ADDRESS_METHOD, vec![StackArg::slice(owner_address.to_boc_base64()?)])
            .await?;
        let wallet = stack_cell_address(&result.stack, 0)?;
        self.jetton_wallet_cache.put(key, wallet.clone());
        Ok(wallet)
    }

    async fn run_get_method(&self, address: &str, method: &str, stack: Vec<StackArg>) -> Result<RunGetMethodResult, SwapperError> {
        let result = self.ton_client.run_get_method(address, method, stack).await.map_err(SwapperError::compute_quote_error)?;
        validate_run_get_method(method, result)
    }

    async fn run_static_get_method(&self, address: &str, method: &str, stack: Vec<StackArg>) -> Result<RunGetMethodResult, SwapperError> {
        let result = self
            .ton_client
            .run_get_method_with_headers(address, method, stack, static_read_cache_headers())
            .await
            .map_err(SwapperError::compute_quote_error)?;
        validate_run_get_method(method, result)
    }
}

fn validate_run_get_method(method: &str, result: RunGetMethodResult) -> Result<RunGetMethodResult, SwapperError> {
    if result.exit_code != 0 {
        return Err(SwapperError::ComputeQuoteError(format!("TON get-method {method} exit code {}", result.exit_code)));
    }
    Ok(result)
}

fn parse_pool_data(result: &RunGetMethodResult) -> Result<PoolData, SwapperError> {
    match result.stack.len() {
        12.. => Ok(PoolData {
            is_locked: stack_num(&result.stack, 0)? != BigUint::from(0u8),
            reserve0: stack_num(&result.stack, 3)?,
            reserve1: stack_num(&result.stack, 4)?,
            token0_wallet: stack_cell_address(&result.stack, 5)?,
            token1_wallet: stack_cell_address(&result.stack, 6)?,
            lp_fee: stack_num_u32(&result.stack, 7)?,
            protocol_fee: stack_num_u32(&result.stack, 8)?,
        }),
        10.. => Ok(PoolData {
            is_locked: false,
            reserve0: stack_num(&result.stack, 0)?,
            reserve1: stack_num(&result.stack, 1)?,
            token0_wallet: stack_cell_address(&result.stack, 2)?,
            token1_wallet: stack_cell_address(&result.stack, 3)?,
            lp_fee: stack_num_u32(&result.stack, 4)?,
            protocol_fee: stack_num_u32(&result.stack, 5)?,
        }),
        _ => Err(SwapperError::ComputeQuoteError("STON.fi get_pool_data returned truncated stack".into())),
    }
}

fn stack_cell_address(stack: &[StackEntry], index: usize) -> Result<String, SwapperError> {
    let bytes = stack
        .get(index)
        .and_then(StackEntry::as_cell_bytes)
        .ok_or_else(|| SwapperError::ComputeQuoteError("missing TON address stack cell".into()))?;
    Ok(PrimitiveAddress::encode(&Address::from_boc_base64(bytes)?))
}

fn stack_num(stack: &[StackEntry], index: usize) -> Result<BigUint, SwapperError> {
    let value = stack
        .get(index)
        .and_then(StackEntry::as_num)
        .ok_or_else(|| SwapperError::ComputeQuoteError("missing TON number stack entry".into()))?;
    parse_ton_num(value)
}

fn stack_num_u32(stack: &[StackEntry], index: usize) -> Result<u32, SwapperError> {
    stack_num(stack, index)?
        .to_u32()
        .ok_or_else(|| SwapperError::ComputeQuoteError("TON stack number does not fit u32".into()))
}

fn parse_ton_num(value: &str) -> Result<BigUint, SwapperError> {
    if let Some(value) = value.strip_prefix("0x") {
        Ok(BigUint::from_str_radix(value, 16)?)
    } else {
        Ok(BigUint::from_str(value)?)
    }
}
