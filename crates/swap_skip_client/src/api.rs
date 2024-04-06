use crate::models::route::{RouteRequest, RouteWithDataRequest};
use reqwest_enum::{
    http::{HTTPBody, HTTPMethod},
    target::Target,
};

use std::collections::HashMap;

pub enum SkipApi {
    Route(RouteRequest),
    MsgsDirect(RouteWithDataRequest),
}

impl Target for SkipApi {
    fn base_url(&self) -> &'static str {
        "https://api.skip.money/v2"
    }

    fn method(&self) -> HTTPMethod {
        HTTPMethod::POST
    }

    fn path(&self) -> &'static str {
        match self {
            SkipApi::Route(_) => "/fungible/route",
            SkipApi::MsgsDirect(_) => "/fungible/msgs_direct",
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
            SkipApi::Route(quote) => HTTPBody::from(quote),
            SkipApi::MsgsDirect(tx) => HTTPBody::from(tx),
        }
    }
}
