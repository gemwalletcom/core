uniffi::include_scaffolding!("gemstone");

use async_std::future::{pending, timeout};
use std::time::Duration;

#[uniffi::export]
pub fn lib_version() -> String {
    String::from("0.1.0")
}

#[uniffi::export]
pub async fn say_after(ms: u64, who: String) -> String {
    let never = pending::<()>();
    timeout(Duration::from_millis(ms), never).await.unwrap_err();
    format!("Hello, {who}!")
}

#[uniffi::export]
pub fn get_name_by_host(host: String) -> String {
    let name = match host.as_str() {
        "etherscan.io" => "Etherscan",
        "tonscan.org" => "TONScan",
        "solscan.io" => "Solscan",
        "opbnbscan.com" => "opBNBScan",
        "bscscan.com" => "BSCScan",
        "blockchair.com" => "Blockchair",
        "tronscan.org" => "TRONSCAN",
        "basescan.org" => "BaseScan",
        "explorer.sui.io" => "Sui Explorer",
        "suiexplorer.com" => "Sui Explorer",
        "explorer.aptoslabs.com" => "Aptos Explorer",
        "mintscan.io" => "MintScan",
        "www.mintscan.io" => "MintScan",
        _ => "Not implemented",
    };
    name.into()
}
