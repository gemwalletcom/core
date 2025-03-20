use crate::{
    network::{AlienProvider, AlienTarget},
    swapper::{debridge::models::*, SwapperError},
};
use serde::de::DeserializeOwned;
use std::sync::Arc;

const API_BASE_URL: &str = "https://api.dln.trade/v1.0";

pub(crate) struct DeBridgeClient {
    provider: Arc<dyn AlienProvider>,
}

impl DeBridgeClient {
    pub fn new(provider: Arc<dyn AlienProvider>) -> Self {
        Self { provider }
    }

    pub async fn create_order<T: DeserializeOwned>(&self, request: &CreateOrderRequest) -> Result<T, SwapperError> {
        let url = format!("{}/dln/order/create-tx", API_BASE_URL);
        let query = serde_urlencoded::to_string(request).map_err(|e| SwapperError::NetworkError { msg: e.to_string() })?;
        let url = format!("{}?{}", url, query);
        let target = AlienTarget::get(&url);

        let response = self.provider.request(target).await?;
        serde_json::from_slice(&response).map_err(|e| SwapperError::NetworkError { msg: e.to_string() })
    }

    #[allow(unused)]
    pub async fn get_order(&self, order_id: &str) -> Result<CreateOrderResponse, SwapperError> {
        let url = format!("{}/dln/order/{}", API_BASE_URL, order_id);
        let target = AlienTarget::get(&url);

        let response = self.provider.request(target).await?;
        serde_json::from_slice(&response).map_err(|e| SwapperError::NetworkError { msg: e.to_string() })
    }

    pub async fn get_order_status(&self, order_id: &str) -> Result<OrderStatus, SwapperError> {
        let url = format!("{}/dln/order/{}/status", API_BASE_URL, order_id);
        let target = AlienTarget::get(&url);

        let response = self.provider.request(target).await?;
        serde_json::from_slice(&response).map_err(|e| SwapperError::NetworkError { msg: e.to_string() })
    }
}
