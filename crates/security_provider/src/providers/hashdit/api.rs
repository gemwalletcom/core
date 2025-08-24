use reqwest_enum::{
    error::Error,
    http::{HTTPBody, HTTPMethod},
    target::Target,
};

use std::{borrow::Cow, collections::HashMap};

pub enum HashDitApi {
    DetectAddress(String, String),
    DetectToken(String, String),
    DetectURL(String),
}

impl Target for HashDitApi {
    fn base_url(&self) -> Cow<'_, str> {
        "https://api.hashdit.io".into()
    }

    fn method(&self) -> HTTPMethod {
        HTTPMethod::POST
    }

    fn path(&self) -> String {
        "/security-api/public/app/v1/detect".into()
    }

    fn query(&self) -> HashMap<String, String> {
        match self {
            HashDitApi::DetectAddress(_, _) => HashMap::from([("business".to_string(), "gem_wallet_address_detection".to_string())]),
            HashDitApi::DetectToken(_, _) => HashMap::from([("business".to_string(), "gem_wallet_token_detection".to_string())]),
            HashDitApi::DetectURL(_) => HashMap::from([("business".to_string(), "gem_wallet_url_detection".to_string())]),
        }
    }

    fn headers(&self) -> HashMap<String, String> {
        HashMap::from([("Content-Type".to_string(), "application/json;charset=UTF-8".to_string())])
    }

    fn body(&self) -> Result<HTTPBody, Error> {
        match self {
            HashDitApi::DetectAddress(address, chain) => {
                let body = serde_json::json!({
                    "address": address,
                    "chain_id": chain,
                });
                HTTPBody::from(&body).map_err(Error::SerdeJson)
            }
            HashDitApi::DetectToken(token_address, chain) => {
                let body = serde_json::json!({
                    "address": token_address,
                    "chain_id": chain,
                });
                HTTPBody::from(&body).map_err(Error::SerdeJson)
            }
            HashDitApi::DetectURL(url) => {
                let body = serde_json::json!({
                    "url": url,
                });
                HTTPBody::from(&body).map_err(Error::SerdeJson)
            }
        }
    }

    fn authentication(&self) -> Option<reqwest_enum::http::AuthMethod> {
        None
    }
}
