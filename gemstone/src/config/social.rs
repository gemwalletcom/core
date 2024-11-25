#[derive(uniffi::Enum, Clone)]
pub enum SocialUrl {
    X,
    Discord,
    Reddit,
    Telegram,
    GitHub,
    YouTube,
    Facebook,
    Website,
    Coingecko,
}

pub fn get_social_url(item: SocialUrl) -> Option<&'static str> {
    match item {
        SocialUrl::X => Some("https://x.com/GemWalletApp"),
        SocialUrl::Discord => Some("https://discord.gg/aWkq5sj7SY"),
        SocialUrl::Telegram => Some("https://t.me/gemwallet"),
        SocialUrl::GitHub => Some("https://github.com/gemwalletcom"),
        SocialUrl::YouTube => Some("https://www.youtube.com/@gemwallet"),
        SocialUrl::Reddit | SocialUrl::Facebook | SocialUrl::Website | SocialUrl::Coingecko => None,
    }
}

#[uniffi::export]
fn social_url_order(url: SocialUrl) -> i32 {
    match url {
        SocialUrl::X => 110,
        SocialUrl::Discord => 1,
        SocialUrl::Reddit => 60,
        SocialUrl::Telegram => 90,
        SocialUrl::GitHub => 20,
        SocialUrl::YouTube => 30,
        SocialUrl::Facebook => 40,
        SocialUrl::Website => 120,
        SocialUrl::Coingecko => 100,
    }
}
