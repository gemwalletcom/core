#[derive(uniffi::Enum, Clone)]
pub enum RewardsUrl {
    Rewards,
}

const WEBSITE_URL: &str = "https://gemwallet.com";

pub fn get_rewards_url(item: RewardsUrl, locale: Option<String>) -> String {
    let path = match item {
        RewardsUrl::Rewards => "/rewards",
    };

    let website_locale = normalize_locale(locale);
    let locale_prefix = if website_locale.is_empty() || website_locale == "en" {
        String::new()
    } else {
        format!("/{}", website_locale)
    };

    format!("{WEBSITE_URL}{locale_prefix}{path}")
}

fn normalize_locale(locale: Option<String>) -> String {
    let Some(loc) = locale else {
        return String::from("en");
    };

    let lower = loc.to_lowercase();
    let parts: Vec<&str> = lower.split(&['-', '_'][..]).collect();

    match parts.first() {
        Some(&"zh") => {
            // Check for script or region
            if parts.iter().any(|p| p == &"hant" || p == &"tw" || p == &"hk") {
                String::from("zh-tw")
            } else {
                String::from("zh-cn")
            }
        }
        Some(&"pt") => {
            if parts.iter().any(|p| p == &"br") {
                String::from("pt-br")
            } else {
                String::from("pt")
            }
        }
        Some(lang) => String::from(*lang),
        None => String::from("en"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_locale() {
        assert_eq!(normalize_locale(None), "en");
        assert_eq!(normalize_locale(Some("en".to_string())), "en");
        assert_eq!(normalize_locale(Some("ru".to_string())), "ru");
        assert_eq!(normalize_locale(Some("zh-Hans".to_string())), "zh-cn");
        assert_eq!(normalize_locale(Some("zh-Hant".to_string())), "zh-tw");
        assert_eq!(normalize_locale(Some("zh_CN".to_string())), "zh-cn");
        assert_eq!(normalize_locale(Some("zh_TW".to_string())), "zh-tw");
        assert_eq!(normalize_locale(Some("pt-BR".to_string())), "pt-br");
        assert_eq!(normalize_locale(Some("pt_PT".to_string())), "pt");
    }

    #[test]
    fn test_get_rewards_url() {
        assert_eq!(get_rewards_url(RewardsUrl::Rewards, Some("en".to_string())), "https://gemwallet.com/rewards");
        assert_eq!(get_rewards_url(RewardsUrl::Rewards, None), "https://gemwallet.com/rewards");
        assert_eq!(get_rewards_url(RewardsUrl::Rewards, Some("ru".to_string())), "https://gemwallet.com/ru/rewards");
        assert_eq!(get_rewards_url(RewardsUrl::Rewards, Some("zh-Hans".to_string())), "https://gemwallet.com/zh-cn/rewards");
        assert_eq!(get_rewards_url(RewardsUrl::Rewards, Some("zh-Hant".to_string())), "https://gemwallet.com/zh-tw/rewards");
        assert_eq!(get_rewards_url(RewardsUrl::Rewards, Some("pt-BR".to_string())), "https://gemwallet.com/pt-br/rewards");
    }
}
