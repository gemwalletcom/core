use super::keccak::keccak256;
use super::namehash::namehash;
use jsonrpsee::core::{client::ClientT, Error};
use jsonrpsee::http_client::HttpClient;
use serde_json::json;

pub struct Contract {
    pub registry: String,
    pub client: HttpClient,
}

impl Contract {
    pub async fn resolver(&self, name: &str) -> Result<String, Error> {
        let hash = namehash(name);
        let data = encode_resolver(hash);
        let response = self.eth_call(&self.registry, data).await?;
        let result = response.unwrap_or_default().to_string().replace("\"", "");
        if result.is_empty() {
            return Err(Error::Custom(String::from("no resolver set")));
        }
        let addr = self.extract_address(&result);
        Ok(addr)
    }

    pub async fn addr(&self, _resolver: &str, _name: &str, _coin_id: u32) -> Result<String, Error> {
        todo!()
    }

    pub async fn legacy_addr(&self, resolver: &str, name: &str) -> Result<String, Error> {
        let hash = namehash(name);
        let data = encode_legacy_addr(hash);
        let response = self.eth_call(resolver, data).await?;
        let result = response.unwrap_or_default().to_string().replace("\"", "");
        if result.is_empty() {
            return Err(Error::Custom(String::from("no address")));
        }
        Ok(self.extract_address(&result))
    }

    async fn eth_call(&self, to: &str, data: Vec<u8>) -> Result<Option<serde_json::Value>, Error> {
        let parmas = json!({
            "to": to,
            "data": format!("0x{}", hex::encode(data))
        });
        self.client
            .request("eth_call", vec![parmas, json!("latest")])
            .await
    }

    fn extract_address(&self, response: &str) -> String {
        // take last 20 bytes
        let result: Vec<char> = response.chars().collect();
        format!(
            "0x{}",
            String::from_iter(result[result.len() - 40..].into_iter())
        )
    }
}

fn encode_resolver(node: Vec<u8>) -> Vec<u8> {
    let mut data: Vec<u8> = encode_func("resolver(bytes32)");
    data.append(&mut node.clone());
    data
}

fn encode_func(func: &str) -> Vec<u8> {
    let hash = keccak256(func.as_bytes());
    hash[..4].to_vec()
}

#[allow(dead_code)]
fn encode_addr(node: Vec<u8>, coin_id: u64) -> Vec<u8> {
    let mut data: Vec<u8> = encode_func("addr(bytes32,uint256)");
    let coin = encode_coin(coin_id);
    data.append(&mut node.clone());
    data.append(&mut coin.clone());
    data
}

fn encode_legacy_addr(node: Vec<u8>) -> Vec<u8> {
    let mut data: Vec<u8> = encode_func("addr(bytes32)");
    data.append(&mut node.clone());
    data
}

#[allow(dead_code)]
fn encode_coin(coin_id: u64) -> Vec<u8> {
    let mut data = vec![0; 24];
    let int = coin_id.to_be_bytes();
    data.extend_from_slice(&int);
    data
}

#[cfg(test)]
mod test {
    use crate::ens_provider::contract::encode_coin;

    use super::encode_func;
    #[test]
    fn test_encode_func() {
        let cases = vec![
            ("resolver(bytes32)", hex::decode("0178b8bf")),
            ("addr(bytes32,uint256)", hex::decode("f1cb7e06")),
        ];

        for (name, expected) in cases {
            let encoded: &[u8] = &encode_func(name);
            assert_eq!(encoded, expected.unwrap());
        }
    }

    #[test]
    fn test_encode_coin() {
        let cases = vec![
            (
                60u64,
                hex::decode("000000000000000000000000000000000000000000000000000000000000003c"),
            ),
            (
                0,
                hex::decode("0000000000000000000000000000000000000000000000000000000000000000"),
            ),
        ];

        for (coin, expected) in cases {
            let encoded: &[u8] = &&encode_coin(coin);
            assert_eq!(encoded, expected.unwrap());
        }
    }
}
