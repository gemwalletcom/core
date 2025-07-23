#[derive(Debug, Serialize)]
#[typeshare(swift = "CaseIterable, Sendable")]
pub enum WalletConnectBlockchainEthereumMethods {
    #[serde(rename = "chainId")]
    chain_id,
    #[serde(rename = "sign")]
    sign,
    #[serde(rename = "personalSign")]
    personal_sign,
    #[serde(rename = "signTypedData")]
    sign_typed_data,
    #[serde(rename = "signTypedDataV4")]
    sign_typed_data_v4,
    #[serde(rename = "signTransaction")]
    sign_transaction,
    #[serde(rename = "sendTransaction")]
    send_transaction,
    #[serde(rename = "sendRawTransaction")]
    send_raw_transaction,
    #[serde(rename = "switchChain")]
    switch_chain,
    #[serde(rename = "addChain")]
    add_chain,
}

#[derive(Debug, Serialize)]
#[typeshare(swift = "CaseIterable, Sendable")]
pub enum WalletConnectBlockchainSolanaMethods {
    #[serde(rename = "signMessage")]
    sign_message,
    #[serde(rename = "signTransaction")]
    sign_transaction,
    #[serde(rename = "signAndSendTransaction")]
    sign_and_send_transaction,
    #[serde(rename = "signAllTransactions")]
    sign_all_transactions,
}