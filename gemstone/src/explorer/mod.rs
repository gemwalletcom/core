pub fn get_name_by_host(host: String) -> Option<String> {
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
