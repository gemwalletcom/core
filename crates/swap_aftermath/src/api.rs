use crate::models::{TradeQuote, TradeTx};
use reqwest_enum::{
    http::{HTTPBody, HTTPMethod},
    target::Target,
};

use std::collections::HashMap;

pub enum AftermathApi {
    Quote(TradeQuote),
    Tx(TradeTx),
}

impl Target for AftermathApi {
    fn base_url(&self) -> &'static str {
        "https://aftermath.finance/api"
    }

    fn method(&self) -> HTTPMethod {
        HTTPMethod::POST
    }

    fn path(&self) -> &'static str {
        match self {
            AftermathApi::Quote(_) => "/router/trade-route",
            AftermathApi::Tx(_) => "/router/transactions/trade-base64",
        }
    }

    fn query(&self) -> std::collections::HashMap<&'static str, &'static str> {
        HashMap::default()
    }

    fn headers(&self) -> std::collections::HashMap<&'static str, &'static str> {
        let mut headers = HashMap::new();
        headers.insert("Content-Type", "application/json");
        headers
    }

    fn body(&self) -> HTTPBody {
        match self {
            AftermathApi::Quote(quote) => HTTPBody::from(quote),
            AftermathApi::Tx(tx) => HTTPBody::from(tx),
        }
    }
}
