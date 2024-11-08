use crate::network::{AlienHttpMethod, AlienProvider, AlienTarget};
use crate::swapper::models::SwapperError;
use crate::swapper::thorchain::model::{QuoteSwapRequest, QuoteSwapResponse};
use primitives::AssetId;
use std::sync::Arc;

use super::model::ThorChainAsset;

#[derive(Debug)]
pub struct ThorChainSwapClient {
    provider: Arc<dyn AlienProvider>,
}

impl ThorChainSwapClient {
    pub fn new(provider: Arc<dyn AlienProvider>) -> Self {
        Self { provider }
    }

    pub async fn get_quote(
        &self,
        endpoint: &str,
        from_asset: AssetId,
        to_asset: AssetId,
        value: String,
        affiliate: String,
        affiliate_bps: i64,
    ) -> Result<QuoteSwapResponse, SwapperError> {
        let from_asset = ThorChainAsset::from_chain(&from_asset.chain).ok_or(SwapperError::NotSupportedChain)?;
        let to_asset = ThorChainAsset::from_chain(&to_asset.chain).ok_or(SwapperError::NotSupportedChain)?;
        let params = QuoteSwapRequest {
            from_asset: from_asset.short_name().to_string(),
            to_asset: to_asset.short_name().to_string(),
            amount: value,
            affiliate,
            affiliate_bps,
        };
        let query = serde_urlencoded::to_string(params).unwrap();
        let url = format!("{}{}?{}", endpoint, "/thorchain/quote/swap", query);

        let target = AlienTarget {
            url,
            method: AlienHttpMethod::Get,
            headers: None,
            body: None,
        };

        let data = self
            .provider
            .request(target)
            .await
            .map_err(|err| SwapperError::NetworkError { msg: err.to_string() })?;

        let result: QuoteSwapResponse = serde_json::from_slice(&data).map_err(|err| SwapperError::NetworkError { msg: err.to_string() })?;

        Ok(result)
    }

    // https://dev.thorchain.org/concepts/memos.html#swap
    pub fn get_memo(to_asset: AssetId, destination_address: String, fee_address: String, bps: u32) -> Option<String> {
        let chain = ThorChainAsset::from_chain(&to_asset.clone().chain)?;
        Some(format!("=:{}:{}::{}:{}", chain.short_name(), destination_address, fee_address, bps))
    }
}

#[cfg(test)]
mod tests {
    use primitives::Chain;

    use super::*;

    #[tokio::test]
    async fn test_get_memo() {
        let destination_address = "0x1234567890abcdef".to_string();
        let fee_address = "0xabcdef1234567890".to_string();
        let bps = 50;

        assert_eq!(
            ThorChainSwapClient::get_memo(Chain::SmartChain.as_asset_id(), destination_address.clone(), fee_address.clone(), bps),
            Some("=:s:0x1234567890abcdef::0xabcdef1234567890:50".into())
        );
        assert_eq!(
            ThorChainSwapClient::get_memo(Chain::Ethereum.as_asset_id(), destination_address.clone(), fee_address.clone(), bps),
            Some("=:e:0x1234567890abcdef::0xabcdef1234567890:50".into())
        );
        assert_eq!(
            ThorChainSwapClient::get_memo(Chain::Doge.as_asset_id(), destination_address.clone(), fee_address.clone(), bps),
            Some("=:d:0x1234567890abcdef::0xabcdef1234567890:50".into())
        );
    }
}
