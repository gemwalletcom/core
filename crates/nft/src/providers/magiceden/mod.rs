pub mod evm;
pub mod solana;

pub use evm::client::MagicEdenEvmClient;
pub use solana::client::MagicEdenSolanaClient;

use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};

pub const BASE_URL: &str = "https://api-mainnet.magiceden.dev";

pub fn create_client(api_key: &str) -> reqwest::Client {
    let mut headers = HeaderMap::new();
    let auth_value = format!("Bearer {}", api_key);
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_value).unwrap());
    reqwest::Client::builder().default_headers(headers).build().unwrap()
}
