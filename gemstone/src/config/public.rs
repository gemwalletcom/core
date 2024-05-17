#[derive(uniffi::Enum, Clone)]
pub enum PublicUrl {
    Website,
    Assets,
    PrivacyPolicy,
    TermsOfService,
    CodebaseIos,
    CodebaseAndroid,
    AppStore,
    PlayStore,
    APK,
}

pub fn get_public_url(item: PublicUrl) -> &'static str {
    match item {
        PublicUrl::Website => "https://gemwallet.com",
        PublicUrl::Assets => "https://assets.gemwallet.com",
        PublicUrl::PrivacyPolicy => "https://gemwallet.com/privacy",
        PublicUrl::TermsOfService => "https://gemwallet.com/terms",
        PublicUrl::CodebaseIos => "https://github.com/gemwalletcom/gem-ios/",
        PublicUrl::CodebaseAndroid => "https://github.com/gemwalletcom/gem-android/",
        PublicUrl::AppStore => "https://apps.apple.com/app/apple-store/id6448712670",
        PublicUrl::PlayStore => {
            "https://play.google.com/store/apps/details?id=com.gemwallet.android"
        }
        PublicUrl::APK => "https://apk.gemwallet.com/gem_wallet_latest.apk",
    }
}
