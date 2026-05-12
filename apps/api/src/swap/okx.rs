use primitives::swap::{ProxyQuote, ProxyQuoteRequest, SwapQuoteData};
use rocket::serde::json::Json;
use swapper::{
    RpcClient,
    okx::OkxProvider,
    proxy::ProxyResponse,
};

#[rocket::post("/swaps/providers/okx/quote", data = "<body>")]
pub async fn post_okx_quote(body: Json<ProxyQuoteRequest>, provider: &rocket::State<OkxProvider<RpcClient>>) -> Json<ProxyResponse<ProxyQuote>> {
    Json(provider.get_quote(body.into_inner()).await.into())
}

#[rocket::post("/swaps/providers/okx/quote_data", data = "<body>")]
pub async fn post_okx_quote_data(body: Json<ProxyQuote>, provider: &rocket::State<OkxProvider<RpcClient>>) -> Json<ProxyResponse<SwapQuoteData>> {
    Json(provider.get_quote_data(body.into_inner()).await.into())
}
