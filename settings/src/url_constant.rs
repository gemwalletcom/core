use typeshare::typeshare;

#[typeshare]
pub enum URLConstant {
    #[serde(rename = "https://gemwallet.com")]
    Website = "https://gemwallet.com",
    
    #[serde(rename = "https://assets.gemwallet.com")]
    Assets = "https://assets.gemwallet.com",

    #[serde(rename = "https://gemwallet.com/privacy")]
    Privacy = "https://gemwallet.com/privacy",

    #[serde(rename = "https://gemwallet.com/terms")]
    TermsOfService = "https://gemwallet.com/terms",

    #[serde(rename = "https://github.com/gemwalletcom/wallet")]
    Code = "https://github.com/gemwalletcom/wallet",

    #[serde(rename = "https://apps.apple.com/app/apple-store/id6448712670")]
    AppStore = "https://apps.apple.com/app/apple-store/id6448712670",

    #[serde(rename = "https://play.google.com/store/apps/details?id=com.gemwallet.android")]
    GooglePlay = "https://play.google.com/store/apps/details?id=com.gemwallet.android",
}
