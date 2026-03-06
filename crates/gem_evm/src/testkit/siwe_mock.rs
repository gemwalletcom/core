pub fn mock_siwe_message(domain: &str, chain_id: u32) -> String {
    [
        &format!("{domain} wants you to sign in with your Ethereum account:"),
        "0x9EdcF9Ff72088DB8130C2512E5B4D3b5F34cEaF4",
        "",
        &format!("URI: https://{domain}"),
        "Version: 1",
        &format!("Chain ID: {chain_id}"),
        "Nonce: gmdhs9w9yfrl2kf2",
        "Issued At: 2026-03-06T01:56:42.927Z",
    ]
    .join("\n")
}

pub fn mock_siwe_message_hex(domain: &str, chain_id: u32) -> String {
    format!("0x{}", hex::encode(mock_siwe_message(domain, chain_id)))
}
