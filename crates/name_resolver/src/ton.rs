use crate::codec::Codec;
use crate::{client::NameClient, model::NameQuery, ton_codec};
use async_trait::async_trait;
use primitives::{Chain, NameProvider};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;

pub struct TONClient {
    url: String,
    client: Client,
}

impl TONClient {
    pub fn new(url: String) -> Self {
        let client = Client::new();
        Self { url, client }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct DnsRecord {
    dns_wallet: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct DnsRecordsResponse {
    records: Vec<DnsRecord>,
}

#[async_trait]
impl NameClient for TONClient {
    fn provider(&self) -> NameProvider {
        NameProvider::Ton
    }

    async fn resolve(&self, query: &NameQuery, _chain: Chain) -> Result<String, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/v3/dns/records", self.url);
        let response = self
            .client
            .get(&url)
            .query(&[("domain", query.domain.as_str()), ("limit", "1")])
            .send()
            .await?
            .error_for_status()?
            .json::<DnsRecordsResponse>()
            .await?;
        let address = response.records.first().and_then(|record| record.dns_wallet.as_deref()).ok_or("missing TON DNS wallet")?;

        // always encode as Bounceable address
        ton_codec::TonCodec::encode(address.as_bytes().to_vec())
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["ton"]
    }

    fn chains(&self) -> Vec<Chain> {
        vec![Chain::Ton]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoding() {
        let raw = "0:8e874b7ad9bbebbfc48810b8939c98f50580246f19982040dbcb253c4c3daf78";
        let address = ton_codec::TonCodec::encode(raw.as_bytes().to_vec()).unwrap();

        assert_eq!(address, "EQCOh0t62bvrv8SIELiTnJj1BYAkbxmYIEDbyyU8TD2veND8");
    }

    #[test]
    fn test_dns_records_response() {
        let response: DnsRecordsResponse = serde_json::from_str(include_str!("../testdata/ton_dns_records_response.json")).unwrap();
        let address = response.records.first().unwrap().dns_wallet.as_deref().unwrap();

        assert_eq!(
            ton_codec::TonCodec::encode(address.as_bytes().to_vec()).unwrap(),
            "EQAzoUpalAaXnVm5MoiYWRZguLFzY0KxFjLv3MkRq5BXzyiQ"
        );
    }
}
