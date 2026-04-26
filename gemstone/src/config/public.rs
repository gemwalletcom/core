use primitives::GEM_ANDROID_PACKAGE_ID;

#[derive(uniffi::Enum, Clone)]
pub enum PublicUrl {
    Website,
    Assets,
    PrivacyPolicy,
    TermsOfService,
    Support,
    CodebaseIos,
    CodebaseAndroid,
    AppStore,
    PlayStore,
    APK,
}

pub const ASSETS_URL: &str = "https://assets.gemwallet.com";
pub const NFT_ASSETS_URL: &str = "https://assets.gemwallet.com/nft";

pub fn get_public_url(item: PublicUrl) -> String {
    match item {
        PublicUrl::Website => "https://gemwallet.com".to_string(),
        PublicUrl::Assets => ASSETS_URL.to_string(),
        PublicUrl::PrivacyPolicy => "https://gemwallet.com/privacy".to_string(),
        PublicUrl::TermsOfService => "https://gemwallet.com/terms".to_string(),
        PublicUrl::Support => "https://gemwallet.com/support".to_string(),
        PublicUrl::CodebaseIos => "https://github.com/gemwalletcom/gem-ios/".to_string(),
        PublicUrl::CodebaseAndroid => "https://github.com/gemwalletcom/gem-android/".to_string(),
        PublicUrl::AppStore => "https://apps.apple.com/app/apple-store/id6448712670".to_string(),
        PublicUrl::PlayStore => format!("https://play.google.com/store/apps/details?id={GEM_ANDROID_PACKAGE_ID}"),
        PublicUrl::APK => "https://apk.gemwallet.com/gem_wallet_latest.apk".to_string(),
    }
}
