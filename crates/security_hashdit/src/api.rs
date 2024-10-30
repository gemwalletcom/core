use reqwest_enum::{
    http::{HTTPBody, HTTPMethod},
    target::Target,
};

use std::{collections::HashMap, time::Duration};

pub enum HashDitApi {
    DetectAddress(String, String),
    DetectURL(String),
}

impl Target for HashDitApi {
    fn base_url(&self) -> &'static str {
        "https://api.hashdit.io"
    }

    fn method(&self) -> HTTPMethod {
        HTTPMethod::POST
    }

    fn path(&self) -> String {
        "/security-api/public/app/v1/detect".into()
    }

    fn query(&self) -> HashMap<&'static str, &'static str> {
        let mut query = HashMap::new();
        match self {
            HashDitApi::DetectAddress(_, _) => {
                query.insert("business", "gem_wallet_address_detection");
            }
            HashDitApi::DetectURL(_) => {
                query.insert("business", "gem_wallet_url_detection");
            }
        }
        query
    }

    fn headers(&self) -> HashMap<&'static str, &'static str> {
        let mut headers = HashMap::new();
        headers.insert("Content-Type", "application/json;charset=UTF-8");
        headers
    }

    fn body(&self) -> HTTPBody {
        match self {
            HashDitApi::DetectAddress(address, chain) => {
                let body = serde_json::json!({
                    "address": address,
                    "chain_id": chain,
                });
                HTTPBody::from(&body)
            }
            HashDitApi::DetectURL(url) => {
                let body = serde_json::json!({
                    "url": url,
                });
                HTTPBody::from(&body)
            }
        }
    }

    fn timeout(&self) -> Option<Duration> {
        None
    }

    fn authentication(&self) -> Option<reqwest_enum::http::AuthMethod> {
        None
    }
}
