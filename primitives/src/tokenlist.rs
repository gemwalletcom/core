#[typeshare]
struct TokenList {
    version: i32,
    assets: Vec<TokenListAsset>,
}

#[typeshare]
struct TokenListAsset {
    chain: Chain,
    #[serde(rename = "tokenId")]
    token_id: String,
    name: String,
    symbol: String,
    #[serde(rename = "type")]
    asset_type: AssetType,
    decimals: i32,
}

#[typeshare]
struct TokenListChainVersion {
    chain: String,
    version: i32,
}