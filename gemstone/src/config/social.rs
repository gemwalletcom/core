use std::str::FromStr;

use primitives::LinkType;

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
fn link_type_order(link_type: String) -> i32 {
    let link_type = LinkType::from_str(link_type.as_str()).ok();
    match link_type {
        Some(value) => match value {
            LinkType::Website => 120,
            LinkType::X => 110,
            LinkType::Coingecko => 105,
            LinkType::CoinMarketCap => 104,
            LinkType::OpenSea => 103,
            LinkType::MagicEden => 102,
            LinkType::Telegram => 90,
            LinkType::Reddit => 60,
            LinkType::Instagram => 50,
            LinkType::Facebook => 40,
            LinkType::TikTok => 35,
            LinkType::Discord => 1,
            LinkType::GitHub => 20,
            LinkType::YouTube => 30,
        },
        None => 0,
    }
}
