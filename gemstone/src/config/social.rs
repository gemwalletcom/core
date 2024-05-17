#[derive(uniffi::Enum, Clone)]
pub enum SocialUrl {
    X,
    Discord,
    Reddit,
    Telegram,
    GitHub,
    YouTube,
}

pub fn get_social_url(item: SocialUrl) -> &'static str {
    match item {
        SocialUrl::X => "https://x.com/GemWalletApp",
        SocialUrl::Discord => "https://discord.gg/aWkq5sj7SY",
        SocialUrl::Reddit => "https://www.reddit.com/r/GemWalletApp/",
        SocialUrl::Telegram => "https://t.me/gemwallet",
        SocialUrl::GitHub => "https://github.com/gemwalletcom",
        SocialUrl::YouTube => "https://www.youtube.com/@gemwallet",
    }
}
