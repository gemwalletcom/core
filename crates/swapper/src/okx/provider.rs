use super::{
    client::OkxDexClient,
    constants::{BASE_URL, EVM_NATIVE_TOKEN_ADDRESS, SOLANA_NATIVE_TOKEN_ADDRESS, chain_index, dex_ids, evm_gas_limit},
    model::{OkxApiResponse, OkxClientConfig, QuoteData, QuoteParams, SwapParams, TransactionData},
    referral::referrer_wallet_addresses,
};
use crate::{
    SwapperError,
    alien::{RpcClient, RpcProvider},
    approval::check_approval_erc20,
    fees::bps_to_percent_string,
    models::ApprovalType,
};
use alloy_primitives::U256;
use gem_client::Client;
use gem_encoding::encode_base64;
use num_bigint::BigUint;
use primitives::{
    Chain, ChainType,
    swap::{ApprovalData, ProxyQuote, ProxyQuoteRequest, QuoteAsset, SwapQuoteData},
};
use serde_json::Value;
use std::{fmt::Debug, str::FromStr, sync::Arc};

const HUNDRED_PERCENT_IN_BPS: u32 = 10_000;
const OKX_GAS_LIMIT_BUFFER_PERCENT: u64 = 50;

#[derive(Debug)]
pub struct OkxProvider<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    client: OkxDexClient<C>,
    rpc_provider: Arc<dyn RpcProvider>,
}

impl OkxProvider<RpcClient> {
    pub fn new(config: OkxClientConfig, rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self::new_with_client(RpcClient::new(BASE_URL.to_string(), rpc_provider.clone()), config, rpc_provider)
    }
}

impl<C> OkxProvider<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    pub fn new_with_client(client: C, config: OkxClientConfig, rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self {
            client: OkxDexClient::new(client, config),
            rpc_provider,
        }
    }

    pub async fn get_quote(&self, request: ProxyQuoteRequest) -> Result<ProxyQuote, SwapperError> {
        let chain = request.from_asset.chain();
        if request.to_asset.chain() != chain {
            return Err(SwapperError::NotSupportedChain);
        }

        let params = QuoteParams {
            chain_index: chain_index(chain).ok_or(SwapperError::NotSupportedChain)?.to_string(),
            amount: request.from_value.clone(),
            from_token_address: asset_to_token_address(&request.from_asset)?,
            to_token_address: asset_to_token_address(&request.to_asset)?,
            slippage_percent: slippage_percent(request.slippage_bps),
            dex_ids: dex_ids(chain),
            fee_percent: bps_to_percent_string(request.referral_bps)?,
        };

        let response = self.client.get_quote(&params).await?;
        let route = first_data(response, "Failed to fetch OKX quote")?;

        let output_min_value = output_min_value(&route.to_token_amount, request.slippage_bps)?;
        let route_data = serde_json::to_value(&route)?;

        Ok(ProxyQuote {
            output_value: route.to_token_amount.clone(),
            output_min_value,
            route_data,
            eta_in_seconds: 0,
            quote: request,
        })
    }

    pub async fn get_quote_data(&self, quote: ProxyQuote) -> Result<SwapQuoteData, SwapperError> {
        let route: QuoteData = serde_json::from_value(quote.route_data.clone()).map_err(|_| SwapperError::InvalidRoute)?;
        let request = &quote.quote;
        let chain = request.from_asset.chain();
        let is_token_swap = chain.chain_type() == ChainType::Ethereum && request.from_asset.asset_id().token_id.is_some();
        let params = build_swap_params(request, &route, chain, is_token_swap)?;

        let response = self.client.get_swap_data(&params).await?;
        let swap_data = first_data(response, "Failed to fetch OKX quote data")?;
        let tx = swap_data.tx;
        if tx.data.is_empty() {
            return Err(SwapperError::InvalidRoute);
        }

        match chain.chain_type() {
            ChainType::Ethereum => self.build_evm_quote_data(&tx, &request.from_asset, &request.from_value, chain, &request.from_address).await,
            ChainType::Solana => build_solana_quote_data(&tx),
            _ => Err(SwapperError::NotSupportedChain),
        }
    }

    async fn build_evm_quote_data(&self, tx: &TransactionData, from_asset: &QuoteAsset, from_value: &str, chain: Chain, owner: &str) -> Result<SwapQuoteData, SwapperError> {
        let approval = self.build_evm_approval(from_asset, tx.signature_data.as_deref(), from_value, chain, owner).await?;
        let gas_limit = approval.is_some().then(|| apply_gas_multiplier_or_default(&tx.gas, chain));
        let value = if tx.value.is_empty() { "0".to_string() } else { tx.value.clone() };
        Ok(SwapQuoteData::new_contract(tx.to.clone(), value, tx.data.clone(), approval, gas_limit))
    }

    async fn build_evm_approval(
        &self,
        from_asset: &QuoteAsset,
        signature_data: Option<&[String]>,
        from_value: &str,
        chain: Chain,
        owner: &str,
    ) -> Result<Option<ApprovalData>, SwapperError> {
        let Some(token) = from_asset.asset_id().token_id else {
            return Ok(None);
        };
        let Some(spender) = extract_spender(signature_data) else {
            return Ok(None);
        };
        let amount = U256::from_str(from_value)?;
        match check_approval_erc20(owner.to_string(), token, spender, amount, self.rpc_provider.clone(), &chain).await? {
            ApprovalType::Approve(data) => Ok(Some(data)),
            _ => Ok(None),
        }
    }
}

fn first_data<T>(response: OkxApiResponse<T>, fallback: &str) -> Result<T, SwapperError> {
    if response.code != "0" {
        let message = if response.msg.is_empty() { fallback.to_string() } else { response.msg };
        return Err(SwapperError::ComputeQuoteError(message));
    }
    response.data.into_iter().next().ok_or(SwapperError::NoQuoteAvailable)
}

fn asset_to_token_address(asset: &QuoteAsset) -> Result<String, SwapperError> {
    let asset_id = asset.asset_id();
    if asset_id.chain == Chain::Solana {
        return Ok(asset_id.token_id.unwrap_or_else(|| SOLANA_NATIVE_TOKEN_ADDRESS.to_string()));
    }
    if asset_id.chain.chain_type() == ChainType::Ethereum {
        return Ok(asset_id.token_id.unwrap_or_else(|| EVM_NATIVE_TOKEN_ADDRESS.to_string()));
    }
    Err(SwapperError::NotSupportedChain)
}

fn slippage_percent(slippage_bps: u32) -> String {
    let bps = if slippage_bps == 0 { 100 } else { slippage_bps.min(100) };
    bps_to_percent_string(bps).unwrap_or_else(|_| "1".to_string())
}

fn max_auto_slippage_percent(slippage_bps: u32) -> Option<String> {
    if slippage_bps == 0 {
        return None;
    }
    bps_to_percent_string(slippage_bps.saturating_mul(2)).ok()
}

fn build_swap_params(request: &ProxyQuoteRequest, route: &QuoteData, chain: Chain, approve_transaction: bool) -> Result<SwapParams, SwapperError> {
    let referrers = referrer_wallet_addresses(&request.from_asset, &request.to_asset, chain);
    Ok(SwapParams {
        chain_index: chain_index(chain).ok_or(SwapperError::NotSupportedChain)?.to_string(),
        amount: request.from_value.clone(),
        from_token_address: route.from_token.token_contract_address.clone(),
        to_token_address: route.to_token.token_contract_address.clone(),
        user_wallet_address: request.from_address.clone(),
        approve_transaction: approve_transaction.then_some(true),
        approve_amount: approve_transaction.then(|| request.from_value.clone()),
        slippage_percent: Some(slippage_percent(request.slippage_bps)),
        auto_slippage: Some(true),
        max_auto_slippage_percent: max_auto_slippage_percent(request.slippage_bps),
        dex_ids: dex_ids(chain),
        fee_percent: bps_to_percent_string(request.referral_bps)?,
        from_token_referrer_wallet_address: referrers.from_token,
        to_token_referrer_wallet_address: referrers.to_token,
    })
}

fn output_min_value(to_token_amount: &str, slippage_bps: u32) -> Result<String, SwapperError> {
    let amount = BigUint::from_str(to_token_amount)?;
    let bps = if slippage_bps == 0 { 100 } else { slippage_bps };
    let remaining = HUNDRED_PERCENT_IN_BPS.saturating_sub(bps.min(HUNDRED_PERCENT_IN_BPS));
    let result = (amount * BigUint::from(remaining)) / BigUint::from(HUNDRED_PERCENT_IN_BPS);
    Ok(result.to_string())
}

fn extract_spender(signature_data: Option<&[String]>) -> Option<String> {
    signature_data
        .unwrap_or(&[])
        .iter()
        .filter_map(|s| serde_json::from_str::<Value>(s).ok())
        .find_map(|v| v.get("approveContract").and_then(|c| c.as_str()).map(str::to_owned))
        .filter(|s| !s.is_empty())
}

fn apply_gas_multiplier_or_default(gas: &str, chain: Chain) -> String {
    match gas.parse::<u64>() {
        Ok(value) if value > 0 => value.saturating_mul(100 + OKX_GAS_LIMIT_BUFFER_PERCENT).div_ceil(100).to_string(),
        _ => evm_gas_limit(chain).to_string(),
    }
}

fn build_solana_quote_data(tx: &TransactionData) -> Result<SwapQuoteData, SwapperError> {
    let bytes = bs58::decode(&tx.data)
        .into_vec()
        .map_err(|err| SwapperError::TransactionError(format!("invalid swap tx data: {err}")))?;
    Ok(SwapQuoteData::new_contract(tx.to.clone(), "0".to_string(), encode_base64(&bytes), None, None))
}

#[cfg(test)]
mod tests {
    use super::super::model::TokenInfo;
    use super::*;
    use crate::fees::default_referral_address;
    use primitives::{
        AssetId,
        asset_constants::{ETHEREUM_USDC_ASSET_ID, ETHEREUM_USDC_TOKEN_ID, SMARTCHAIN_CAKE_TOKEN_ID, SOLANA_USDC_ASSET_ID, SOLANA_USDC_TOKEN_ID},
    };

    fn quote_asset(id: &str) -> QuoteAsset {
        quote_asset_with_symbol(id, "")
    }

    fn quote_asset_with_symbol(id: &str, symbol: &str) -> QuoteAsset {
        QuoteAsset {
            id: id.to_string(),
            symbol: symbol.to_string(),
            decimals: 18,
        }
    }

    fn proxy_request_with_assets(from_asset: QuoteAsset, to_asset: QuoteAsset, slippage_bps: u32, referral_bps: u32) -> ProxyQuoteRequest {
        ProxyQuoteRequest {
            from_address: "0xabc".to_string(),
            to_address: "0xabc".to_string(),
            from_asset,
            to_asset,
            from_value: "1000000000000000000".to_string(),
            referral_bps,
            slippage_bps,
            use_max_amount: false,
        }
    }

    fn proxy_request(from_id: &str, to_id: &str, slippage_bps: u32, referral_bps: u32) -> ProxyQuoteRequest {
        proxy_request_with_assets(quote_asset(from_id), quote_asset(to_id), slippage_bps, referral_bps)
    }

    fn quote_data(from_token: &str, to_token: &str) -> QuoteData {
        QuoteData {
            from_token: TokenInfo {
                token_contract_address: from_token.to_string(),
            },
            to_token: TokenInfo {
                token_contract_address: to_token.to_string(),
            },
            to_token_amount: "200".to_string(),
        }
    }

    #[test]
    fn test_slippage_percent() {
        assert_eq!(slippage_percent(0), "1");
        assert_eq!(slippage_percent(10), "0.1");
        assert_eq!(slippage_percent(50), "0.5");
        assert_eq!(slippage_percent(100), "1");
        assert_eq!(slippage_percent(500), "1");
    }

    #[test]
    fn test_asset_to_token_address() {
        let sol = AssetId::from_chain(Chain::Solana).to_string();
        let eth = AssetId::from_chain(Chain::Ethereum).to_string();
        assert_eq!(asset_to_token_address(&quote_asset(&sol)).unwrap(), SOLANA_NATIVE_TOKEN_ADDRESS);
        assert_eq!(asset_to_token_address(&quote_asset(&eth)).unwrap(), EVM_NATIVE_TOKEN_ADDRESS);
        assert_eq!(asset_to_token_address(&quote_asset(&ETHEREUM_USDC_ASSET_ID.to_string())).unwrap(), ETHEREUM_USDC_TOKEN_ID);
    }

    #[test]
    fn test_output_min_value() {
        assert_eq!(output_min_value("1000", 0).unwrap(), "990");
        assert_eq!(output_min_value("1000", 100).unwrap(), "990");
        assert_eq!(output_min_value("1000", 300).unwrap(), "970");
        assert_eq!(output_min_value("10000000000000000000", 50).unwrap(), "9950000000000000000");
        assert_eq!(output_min_value("1000", 11_000).unwrap(), "0");
    }

    #[test]
    fn test_first_data() {
        let err_response: OkxApiResponse<QuoteData> = OkxApiResponse {
            code: "50011".to_string(),
            msg: "Request frequency too high".to_string(),
            data: vec![],
        };
        assert!(matches!(first_data(err_response, "fallback"), Err(SwapperError::ComputeQuoteError(msg)) if msg == "Request frequency too high"));

        let empty_response: OkxApiResponse<QuoteData> = OkxApiResponse {
            code: "0".to_string(),
            msg: String::new(),
            data: vec![],
        };
        assert!(matches!(first_data(empty_response, "fallback"), Err(SwapperError::NoQuoteAvailable)));
    }

    #[test]
    fn test_apply_gas_multiplier_or_default() {
        assert_eq!(apply_gas_multiplier_or_default("200000", Chain::Ethereum), "300000");
        assert_eq!(apply_gas_multiplier_or_default("", Chain::Ethereum), "920000");
        assert_eq!(apply_gas_multiplier_or_default("0", Chain::Ethereum), "920000");
        assert_eq!(apply_gas_multiplier_or_default("0", Chain::Mantle), "2000000000");
    }

    #[test]
    fn test_extract_spender() {
        let valid = vec![r#"{"approveContract":"0x40aA958dd87FC8305b97f2BA922CDdCa374bcD7f"}"#.to_string()];
        assert_eq!(extract_spender(Some(&valid)).unwrap(), "0x40aA958dd87FC8305b97f2BA922CDdCa374bcD7f");
        assert!(extract_spender(Some(&["not json".to_string()])).is_none());
        assert!(extract_spender(Some(&[r#"{"approveContract":""}"#.to_string()])).is_none());
        assert!(extract_spender(None).is_none());
    }

    #[test]
    fn test_build_solana_quote_data() {
        let tx = TransactionData {
            data: "Cn8eVZg".to_string(),
            to: "ToAddr".to_string(),
            value: String::new(),
            gas: String::new(),
            signature_data: None,
        };
        let data = build_solana_quote_data(&tx).unwrap();
        assert_eq!(data.data, "aGVsbG8=");
        assert_eq!(data.value, "0");
        assert_eq!(data.to, "ToAddr");
        assert!(data.gas_limit.is_none());
        assert!(data.approval.is_none());

        let invalid = TransactionData {
            data: "0OIl".to_string(),
            to: String::new(),
            value: String::new(),
            gas: String::new(),
            signature_data: None,
        };
        assert!(matches!(build_solana_quote_data(&invalid), Err(SwapperError::TransactionError(_))));
    }

    #[test]
    fn test_build_swap_params() {
        let eth = AssetId::from_chain(Chain::Ethereum).to_string();
        let evm_request = proxy_request(&ETHEREUM_USDC_ASSET_ID.to_string(), &eth, 100, 50);
        let evm_route = quote_data(ETHEREUM_USDC_TOKEN_ID, EVM_NATIVE_TOKEN_ADDRESS);
        let evm_params = build_swap_params(&evm_request, &evm_route, Chain::Ethereum, true).unwrap();
        assert_eq!(evm_params.chain_index, "1");
        assert_eq!(evm_params.approve_transaction, Some(true));
        assert_eq!(evm_params.approve_amount.as_deref(), Some("1000000000000000000"));
        assert_eq!(evm_params.fee_percent, "0.5");
        assert_eq!(evm_params.auto_slippage, Some(true));
        assert_eq!(evm_params.dex_ids, None);
        assert_eq!(evm_params.max_auto_slippage_percent.as_deref(), Some("2"));
        assert!(evm_params.to_token_referrer_wallet_address.is_some());
        assert!(evm_params.from_token_referrer_wallet_address.is_none());

        let bnb = AssetId::from_chain(Chain::SmartChain).to_string();
        let cake = AssetId::from_token(Chain::SmartChain, SMARTCHAIN_CAKE_TOKEN_ID).to_string();
        let bsc_request = proxy_request_with_assets(quote_asset_with_symbol(&bnb, "BNB"), quote_asset_with_symbol(&cake, "CAKE"), 100, 70);
        let bsc_route = quote_data(EVM_NATIVE_TOKEN_ADDRESS, SMARTCHAIN_CAKE_TOKEN_ID);
        let bsc_params = build_swap_params(&bsc_request, &bsc_route, Chain::SmartChain, false).unwrap();
        let evm_referrer = default_referral_address(Chain::SmartChain);
        assert_eq!(bsc_params.from_token_referrer_wallet_address.as_deref(), Some(evm_referrer.as_str()));
        assert_eq!(bsc_params.to_token_referrer_wallet_address, None);

        let sol = AssetId::from_chain(Chain::Solana).to_string();
        let sol_request = proxy_request(&sol, &SOLANA_USDC_ASSET_ID.to_string(), 300, 50);
        let sol_route = quote_data(SOLANA_NATIVE_TOKEN_ADDRESS, SOLANA_USDC_TOKEN_ID);
        let sol_params = build_swap_params(&sol_request, &sol_route, Chain::Solana, false).unwrap();
        assert_eq!(sol_params.chain_index, "501");
        assert!(sol_params.approve_transaction.is_none());
        assert!(sol_params.approve_amount.is_none());
        assert!(sol_params.dex_ids.is_some());
        assert_eq!(sol_params.fee_percent, "0.5");
        assert!(sol_params.from_token_referrer_wallet_address.is_some());
        assert!(sol_params.to_token_referrer_wallet_address.is_none());
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::alien::reqwest_provider::NativeProvider;
    use primitives::{AssetId, asset_constants::SOLANA_USDC_ASSET_ID, swap::QuoteAsset, testkit::signer_mock::TEST_SOLANA_SENDER};

    fn okx_provider() -> OkxProvider<RpcClient> {
        let settings = settings::testkit::get_test_settings();
        let config = OkxClientConfig {
            api_key: settings.swap.okx.key.public,
            secret_key: settings.swap.okx.key.secret,
            passphrase: settings.swap.okx.passphrase,
            project: settings.swap.okx.project,
        };
        OkxProvider::new(config, Arc::new(NativeProvider::default()))
    }

    #[tokio::test]
    async fn test_okx_fetch_quote_and_quote_data_sol_to_usdc() -> Result<(), SwapperError> {
        let provider = okx_provider();
        let request = ProxyQuoteRequest {
            from_address: TEST_SOLANA_SENDER.to_string(),
            to_address: TEST_SOLANA_SENDER.to_string(),
            from_asset: QuoteAsset::from(AssetId::from_chain(Chain::Solana)),
            to_asset: QuoteAsset::from(SOLANA_USDC_ASSET_ID.clone()),
            from_value: "100000000".to_string(),
            referral_bps: 50,
            slippage_bps: 300,
            use_max_amount: false,
        };

        let quote = provider.get_quote(request).await?;
        assert!(quote.output_value.parse::<u64>().unwrap() > 0);
        assert!(quote.output_min_value.parse::<u64>().unwrap() > 0);

        let quote_data = provider.get_quote_data(quote).await?;
        assert!(!quote_data.to.is_empty());
        assert_eq!(quote_data.value, "0");
        assert!(!quote_data.data.is_empty());
        Ok(())
    }
}
