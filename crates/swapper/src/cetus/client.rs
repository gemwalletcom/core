use super::{
    constants::{BLUEFIN, CETUS, CETUS_DLMM, DEEPBOOK_V3, ROUTER_API_VERSION},
    model::{RouterData, RouterRequest, RouterResponse},
};
use crate::SwapperError;
use gem_client::{ClientBounds, ClientExt, build_path_with_query};

#[derive(Debug)]
pub struct CetusClient<C>
where
    C: ClientBounds,
{
    client: C,
}

impl<C> CetusClient<C>
where
    C: ClientBounds,
{
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_router(&self, from: String, target: String, amount: String) -> Result<RouterData, SwapperError> {
        let request = RouterRequest {
            from,
            target,
            amount,
            by_amount_in: true,
            providers: [CETUS, CETUS_DLMM, DEEPBOOK_V3, BLUEFIN].join(","),
            v: ROUTER_API_VERSION,
        };
        let path = build_path_with_query("/find_routes", &request)?;
        let response: RouterResponse = self.client.get(&path).await?;

        match response {
            RouterResponse::Ok { data } => Ok(data),
            RouterResponse::Err { code, msg } => {
                if code == 5000 || msg.to_ascii_lowercase().contains("liquidity") {
                    return Err(SwapperError::NoQuoteAvailable);
                }
                Err(SwapperError::ComputeQuoteError(msg))
            }
        }
    }
}
