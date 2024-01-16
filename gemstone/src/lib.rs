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
pub fn get_explorer_name_by_host(host: String) -> Option<String> {
    return match host.as_str() {
        "etherscan.io" => Some("Etherscan".into()),
        "tonscan.org" => Some("TONScan".into()),
        "solscan.io" => Some("Solscan".into()),
        "opbnbscan.com" => Some("opBNBScan".into()),
        "bscscan.com" => Some("BSCScan".into()),
        "blockchair.com" => Some("Blockchair".into()),
        "tronscan.org" => Some("TRONSCAN".into()),
        "basescan.org" => Some("BaseScan".into()),
        "explorer.sui.io" | "suiexplorer.com" => Some("Sui Explorer".into()),
        "explorer.aptoslabs.com" => Some("Aptos Explorer".into()),
        "mintscan.io" | "www.mintscan.io" => Some("MintScan".into()),
        _ => None,
    };
}
