use reqwest_enum::{
    http::{HTTPBody, HTTPMethod},
    target::Target,
};

use std::collections::HashMap;

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

    fn path(&self) -> &'static str {
        match self {
            HashDitApi::DetectAddress(_, _) => "/security-api/public/chain/v1/detect/address",
            HashDitApi::DetectURL(_) => "/security-api/public/chain/v1/detect/url",
        }
    }

    fn query(&self) -> HashMap<&'static str, &'static str> {
        HashMap::default()
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
}
