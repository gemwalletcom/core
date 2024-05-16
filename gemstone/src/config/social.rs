#[derive(uniffi::Enum, Clone)]
pub enum SocialItem {
    X,
    Discord,
    Reddit,
    Telegram,
    GitHub,
    YouTube,
}

pub fn get_social_url(item: SocialItem) -> &'static str {
    match item {
        SocialItem::X => "https://x.com/GemWalletApp",
        SocialItem::Discord => "https://discord.gg/aWkq5sj7SY",
        SocialItem::Reddit => "https://www.reddit.com/r/GemWalletApp/",
        SocialItem::Telegram => "https://t.me/gemwallet",
        SocialItem::GitHub => "https://github.com/gemwalletcom",
        SocialItem::YouTube => "https://www.youtube.com/@gemwallet",
    }
}
