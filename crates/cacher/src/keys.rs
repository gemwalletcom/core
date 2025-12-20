const SECONDS_PER_MINUTE: u64 = 60;
const SECONDS_PER_DAY: u64 = 24 * 60 * 60;

pub enum CacheKey<'a> {
    // Referral keys
    ReferralIpCheck(&'a str),
    ReferralDailyLimit(&'a str),
    ReferralWeeklyLimit(&'a str),
    ReferralUseDailyLimit,

    // Device keys
    InactiveDeviceObserver(&'a str),

    // Fetch consumer keys (chain, address)
    FetchCoinAddresses(&'a str, &'a str),
    FetchTokenAddresses(&'a str, &'a str),
    FetchNftAssetsAddresses(&'a str, &'a str),
    FetchAddressTransactions(&'a str, &'a str),

    // Asset keys
    FetchAssets(&'a str),
    PricerCoinInfo(&'a str),

    // Fiat keys
    FiatQuote(&'a str),
    FiatIpCheck(&'a str),

    // Auth keys (device_id, nonce)
    AuthNonce(&'a str, &'a str),
}

impl CacheKey<'_> {
    pub fn key(&self) -> String {
        match self {
            Self::ReferralIpCheck(ip_address) => format!("referral:ip_check:{}", ip_address),
            Self::ReferralDailyLimit(ip_address) => format!("referral:ip_daily:{}", ip_address),
            Self::ReferralWeeklyLimit(ip_address) => format!("referral:ip_weekly:{}", ip_address),
            Self::ReferralUseDailyLimit => "referral:use_daily".to_string(),
            Self::InactiveDeviceObserver(device_id) => format!("device:inactive_observer:{}", device_id),
            Self::FetchCoinAddresses(chain, address) => format!("fetch:coin_addresses:{}:{}", chain, address),
            Self::FetchTokenAddresses(chain, address) => format!("fetch:token_addresses:{}:{}", chain, address),
            Self::FetchNftAssetsAddresses(chain, address) => format!("fetch:nft_assets_addresses:{}:{}", chain, address),
            Self::FetchAddressTransactions(chain, address) => format!("fetch:address_transactions:{}:{}", chain, address),
            Self::FetchAssets(asset_id) => format!("fetch:assets:{}", asset_id),
            Self::PricerCoinInfo(coin_id) => format!("pricer:coin_info:{}", coin_id),
            Self::FiatQuote(quote_id) => format!("fiat:quote:{}", quote_id),
            Self::FiatIpCheck(ip_address) => format!("fiat:ip_check:{}", ip_address),
            Self::AuthNonce(device_id, nonce) => format!("auth:nonce:{}:{}", device_id, nonce),
        }
    }

    pub fn ttl(&self) -> u64 {
        match self {
            Self::ReferralIpCheck(_) => 30 * SECONDS_PER_DAY,
            Self::ReferralDailyLimit(_) => SECONDS_PER_DAY,
            Self::ReferralWeeklyLimit(_) => 7 * SECONDS_PER_DAY,
            Self::ReferralUseDailyLimit => SECONDS_PER_DAY,
            Self::InactiveDeviceObserver(_) => 30 * SECONDS_PER_DAY,
            Self::FetchCoinAddresses(_, _) => 7 * SECONDS_PER_DAY,
            Self::FetchTokenAddresses(_, _) => 30 * SECONDS_PER_DAY,
            Self::FetchNftAssetsAddresses(_, _) => 30 * SECONDS_PER_DAY,
            Self::FetchAddressTransactions(_, _) => 30 * SECONDS_PER_DAY,
            Self::FetchAssets(_) => 30 * SECONDS_PER_DAY,
            Self::PricerCoinInfo(_) => SECONDS_PER_DAY,
            Self::FiatQuote(_) => 15 * SECONDS_PER_MINUTE,
            Self::FiatIpCheck(_) => SECONDS_PER_DAY,
            Self::AuthNonce(_, _) => 5 * SECONDS_PER_MINUTE,
        }
    }
}
