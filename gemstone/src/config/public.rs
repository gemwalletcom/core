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
pub const NFT_ASSETS_URL: &str = "https://nft-assets.gemwallet.com";

pub fn get_public_url(item: PublicUrl) -> &'static str {
    match item {
        PublicUrl::Website => "https://gemwallet.com",
        PublicUrl::Assets => ASSETS_URL,
        PublicUrl::PrivacyPolicy => "https://gemwallet.com/privacy",
        PublicUrl::TermsOfService => "https://gemwallet.com/terms",
        PublicUrl::Support => "https://gemwallet.com/support",
        PublicUrl::CodebaseIos => "https://github.com/gemwalletcom/gem-ios/",
        PublicUrl::CodebaseAndroid => "https://github.com/gemwalletcom/gem-android/",
        PublicUrl::AppStore => "https://apps.apple.com/app/apple-store/id6448712670",
        PublicUrl::PlayStore => "https://play.google.com/store/apps/details?id=com.gemwallet.android",
        PublicUrl::APK => "https://apk.gemwallet.com/gem_wallet_latest.apk",
    }
}
