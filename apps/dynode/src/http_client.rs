use bytes::Bytes;
use http_body_util::Full;
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::{connect::HttpConnector, Client};

pub type HttpClient = Client<HttpsConnector<HttpConnector>, Full<Bytes>>;

pub fn new() -> HttpClient {
    let https = HttpsConnector::new();
    Client::builder(hyper_util::rt::TokioExecutor::new()).build(https)
}

