use crate::swapper::across::asset::AcrossChainAsset;
use crate::swapper::across::chain::AcrossChainName;
use crate::swapper::across::client::AcrossSwapClient;
use crate::{
    network::AlienProvider,
    swapper::{models::*, GemSwapProvider, SwapperError},
};
use async_trait::async_trait;
use gem_evm::uniswap::FeeTier;
use num_bigint::BigInt;
use primitives::Chain;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug)]
pub struct Across {
    pub chain: Chain,
}

impl Default for Across {
    fn default() -> Self {
        Self { chain: Chain::Solana }
    }
}

#[async_trait]
impl GemSwapProvider for Across {
    fn provider(&self) -> SwapProvider {
        SwapProvider::Across
    }

    fn supported_chains(&self) -> Vec<Chain> {
        AcrossChainName::all().iter().map(|name| name.chain()).collect()
    }

    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError> {
        // Prevent swaps on unsupported chains
        if !self.supported_chains().contains(&request.from_asset.chain) {
            return Err(SwapperError::NotSupportedChain);
        }
        let client = AcrossSwapClient::new(provider);
        let from_asset = AcrossChainAsset::from_asset_id(request.clone().from_asset).ok_or(SwapperError::NotSupportedAsset)?;
        let to_asset = AcrossChainAsset::from_asset_id(request.clone().to_asset).ok_or(SwapperError::NotSupportedAsset)?;
        let quote = client
            .get_quote(&self.get_endpoint(), from_asset.clone(), to_asset.clone(), request.clone().value)
            .await?;

        let to_value = self.value_to(request.value.to_string(), to_asset.decimals as i32);

        let quote = SwapQuote {
            from_value: request.clone().value,
            to_value: to_value.to_string(),
            data: SwapProviderData {
                provider: self.provider(),
                routes: vec![SwapRoute {
                    input: request.from_asset.clone(),
                    output: request.to_asset.clone(),
                    gas_estimate: None,
                    route_data: "".to_string(),
                }],
            },
            approval: ApprovalType::None,
            request: request.clone(),
        };

        Ok(quote)
    }

    async fn fetch_quote_data(&self, _quote: &SwapQuote, _provider: Arc<dyn AlienProvider>, _data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        todo!()
    }

    async fn get_transaction_status(&self, _chain: Chain, _transaction_hash: &str, _provider: Arc<dyn AlienProvider>) -> Result<bool, SwapperError> {
        // TODO: the transaction status from the RPC
        Ok(true)
    }
}

impl Across {
    pub fn get_endpoint(&self) -> String {
        "https://app.across.to".into()
    }
    fn value_from(&self, value: String, decimals: i32) -> BigInt {
        let decimals = decimals - 8;
        if decimals > 0 {
            BigInt::from_str(value.as_str()).unwrap() / BigInt::from(10).pow(decimals as u32)
        } else {
            BigInt::from_str(value.as_str()).unwrap() * BigInt::from(10).pow(decimals.unsigned_abs())
        }
    }

    fn value_to(&self, value: String,  decimals: i32) -> BigInt {
        let decimals = decimals - 8;
        if decimals > 0 {
            BigInt::from_str(value.as_str()).unwrap() * BigInt::from(10).pow((decimals).unsigned_abs())
        } else {
            BigInt::from_str(value.as_str()).unwrap() / BigInt::from(10).pow((decimals).unsigned_abs())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_swap_quote_by_input_token() -> Result<(), SwapperError> {
    //     let data = include_str!("test/tick_array_response.json");
    //     let response: JsonRpcResult<ValueResult<Vec<AccountData>>> = serde_json::from_slice(data.as_bytes()).unwrap();
    //     let tick_accounts = response.extract_result().unwrap().value;
    //     let base64_strs: Vec<String> = tick_accounts.iter().map(|x| x.data[0].clone()).collect();
    //     let mut tick_array: Vec<TickArray> = vec![];
    //     for base64_str in base64_strs.iter() {
    //         let tick: TickArray = try_borsh_decode(base64_str).unwrap();
    //         tick_array.push(tick);
    //     }
    //
    //     tick_array.sort_by_key(|x| x.start_tick_index);
    //
    //     let tick_array_facades = tick_array.into_iter().map(|x| TickArrayFacade::from(&x)).collect::<Vec<_>>();
    //
    //     let result: [TickArrayFacade; 5] = std::array::from_fn(|i| tick_array_facades[i]);
    //     let tick_arrays = TickArrays::from(result);
    //
    //     let amount_in = 1000000;
    //     let slippage_bps = 100;
    //     let base64_str = "P5XRDOGAYwkT5EH4ORPKaLBjT7Al/eqohzfoQRDRJV41ezN33e4czf8EAAQAkAEBACUn6rOx9gIAAAAAAAAAAADZ0q3a01wPfgAAAAAAAAAApsj///QCNRYAAAAA7MHhBAAAAAAGm4hX/quBhPtof2NGGMA12sQ53BrrO1WYoPAAAAAAAchN8kM4mDvkqFswl7r0C8lXEQjSiawAs2jfF11Edc96okZrwvdXv2MAAAAAAAAAAMb6evO+2606PWXzaqvJdDGxu+TC0vbg5HymAgNFL11hFl+VcsWpaqUC3VEQVKJqbSWO98HW1sGu4SkZFNxRAjLtNOmyVWgdCwAAAAAAAAAAaZY8ZwAAAAAMANCv64YU2n8Zq6AtQPGMaSWF9lAg387T1eX5qcDE4Q8bkJQIzrVDfhKReyB9qZTQ6FenQB4SLAPfa/fG1/wqvR0xrxfe/zwmhIFgCsr+SxQJjA/hQbf0oc34STRkRAMAAAAAAAAAAAAAAAAAAAAAIxHh3tFPDkQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAC9HTGvF97/PCaEgWAKyv5LFAmMD+FBt/ShzfhJNGREAwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAL0dMa8X3v88JoSBYArK/ksUCYwP4UG39KHN+Ek0ZEQDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
    //     let pool: Whirlpool = try_borsh_decode(base64_str).unwrap();
    //
    //     let quote =
    //         swap_quote_by_input_token(amount_in, true, slippage_bps, (&pool).into(), tick_arrays, None, None).map_err(|c| SwapperError::ComputeQuoteError {
    //             msg: format!("swap_quote_by_input_token error: {:?}", c),
    //         })?;
    //     assert_eq!(quote.token_min_out, 239958);
    //     Ok(())
    // }
    //
    // #[test]
    // fn test_get_tick_array_start_tick_index() {
    //     let tick_current_index = -15865;
    //     let tick_spacing = 4;
    //     let start_index = get_tick_array_start_tick_index(tick_current_index, tick_spacing);
    //
    //     assert_eq!(start_index, -16192);
    //
    //     let pool = Pubkey::from_str("Czfq3xZZDmsdGdUyrNLtRhGc47cXcZtLG4crryfu44zE").unwrap();
    //     let tick_array_address = get_tick_array_address(&pool, start_index).unwrap();
    //
    //     assert_eq!(tick_array_address.0.to_string(), "3M9oTcoC5viBCNuJEKgwCrQDEbE3Rh6CpTGP5C2jGHzU");
    // }
}
