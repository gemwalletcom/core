use super::asset::AcrossChainAsset;
use crate::network::{AlienHttpMethod, AlienProvider, AlienTarget};
use crate::swapper::across::model::{SaggestedFeesRequest, SaggestedFeesResponse};
use crate::swapper::models::SwapperError;
use std::sync::Arc;

#[derive(Debug)]
pub struct AcrossSwapClient {
    provider: Arc<dyn AlienProvider>,
}

impl AcrossSwapClient {
    pub fn new(provider: Arc<dyn AlienProvider>) -> Self {
        Self { provider }
    }

    pub async fn get_quote(
        &self,
        endpoint: &str,
        from_asset: AcrossChainAsset,
        to_asset: AcrossChainAsset,
        value: String,
    ) -> Result<SaggestedFeesResponse, SwapperError> {
        let params = SaggestedFeesRequest {
            input_token: from_asset.token_id.unwrap(),
            amount: value,
            origin_chain_id: from_asset.chain.chain().network_id().to_string(),
            destination_chain_id: to_asset.chain.chain().network_id().to_string(),
            recipient: None,
            message: None,
            relayer: None,
            output_token: to_asset.token_id.unwrap(),
            timestamp: None,
        };
        let query = serde_urlencoded::to_string(params).unwrap();
        let url = format!("{}{}?{}", endpoint, "/api/suggested-fees", query);

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
        let result: SaggestedFeesResponse = serde_json::from_slice(&data).map_err(|err| SwapperError::NetworkError { msg: err.to_string() })?;
        Ok(result)
    }
}
