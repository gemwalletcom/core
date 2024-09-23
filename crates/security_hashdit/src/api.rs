use reqwest_enum::{
    http::{HTTPBody, HTTPMethod},
    target::Target,
};

use std::collections::HashMap;

pub enum HashDitApi {
    DetectAddress,
    DetectURL,
}

impl Target for HashDitApi {
    fn base_url(&self) -> &'static str {
        "https://api.hashdit.io/security-api/public/chain"
    }

    fn method(&self) -> HTTPMethod {
        HTTPMethod::POST
    }

    fn path(&self) -> &'static str {
        match self {
            HashDitApi::DetectAddress => "/v1/detect/address",
            HashDitApi::DetectURL => "/v1/detect/url",
        }
    }

    fn query(&self) -> HashMap<&'static str, &'static str> {
        HashMap::default()
    }

    fn headers(&self) -> HashMap<&'static str, &'static str> {
        let mut headers = HashMap::new();
        headers.insert("Content-Type", "application/json");
        headers
    }

    fn body(&self) -> HTTPBody {
        todo!()
    }
}
