#[derive(uniffi::Enum, Clone)]
pub enum SocialUrl {
    X,
    Discord,
    Reddit,
    Telegram,
    GitHub,
    YouTube,
    Facebook,
    Homepage,
    Coingecko,
}

pub fn get_social_url(item: SocialUrl) -> Option<&'static str> {
    match item {
        SocialUrl::X => Some("https://x.com/GemWalletApp"),
        SocialUrl::Discord => Some("https://discord.gg/aWkq5sj7SY"),
        SocialUrl::Telegram => Some("https://t.me/gemwallet"),
        SocialUrl::GitHub => Some("https://github.com/gemwalletcom"),
        SocialUrl::YouTube => Some("https://www.youtube.com/@gemwallet"),
        SocialUrl::Reddit | SocialUrl::Facebook | SocialUrl::Homepage | SocialUrl::Coingecko => None,
    }
}
