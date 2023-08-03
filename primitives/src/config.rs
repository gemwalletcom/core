#[typeshare(swift="Codable")]
#[serde(rename_all = "camelCase")]
struct ConfigResponse {
    app: ConfigApp,
    versions: ConfigVersions,
}

#[typeshare(swift="Codable")]
struct ConfigApp {
    ios: ConfigIOSApp,
    android: ConfigAndroidApp,
}

#[typeshare(swift="Codable")]
struct ConfigIOSApp {
    version: ConfigAppVersion
}

#[typeshare(swift="Codable")]
struct ConfigAndroidApp {
    version: ConfigAppVersion
}

#[typeshare(swift="Codable")]
struct ConfigAppVersion {
    production: String, 
    beta: String,
    alpha: String,
}

#[typeshare(swift="Codable")]
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigVersions {
    pub nodes: i32,
    pub fiat_assets: i32,
    pub token_lists: i32,
    pub token_lists_chains: Vec<TokenListChainVersion>,
}