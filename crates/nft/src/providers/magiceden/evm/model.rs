use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct TokensResponse {
    pub assets: Vec<TokenAsset>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct TokenAsset {
    pub asset: TokenDetail,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub token_id: String,
    pub collection_id: String,
    pub contract_address: String,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Attribute {
    pub trait_type: String,
    pub value: serde_json::Value,
}

#[derive(Deserialize, Clone, Debug)]
pub struct CollectionsResponse {
    pub collections: Vec<CollectionDetail>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CollectionDetail {
    pub id: String,
    pub name: String,
    pub symbol: Option<String>,
    pub description: Option<String>,
    pub media: Option<CollectionMedia>,
    pub social: Option<CollectionSocial>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CollectionMedia {
    pub url: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CollectionSocial {
    pub discord_url: Option<String>,
    pub website_url: Option<String>,
    pub twitter_url: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TokenDetailResponse {
    pub token: TokenDetail,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TokenDetail {
    pub token_id: String,
    pub name: Option<String>,
    #[serde(rename = "mediaV2")]
    pub media_v2: Option<Media>,
    pub collection_id: String,
    pub owner: Option<String>,
    pub attributes: Option<Vec<Attribute>>,
    pub description: Option<String>,
    pub standard: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Media {
    pub main: Option<MediaMain>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct MediaMain {
    pub uri: Option<String>,
}
