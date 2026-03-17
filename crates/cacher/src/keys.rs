const SECONDS_PER_MINUTE: u64 = 60;
const SECONDS_PER_DAY: u64 = 24 * 60 * 60;

pub enum CacheKey<'a> {
    // Referral keys
    ReferralIpCheck(&'a str),

    // Username keys
    UsernameCreationPerIp(&'a str),
    UsernameCreationPerDevice(i32),
    UsernameCreationGlobalDaily,
    UsernameCreationPerCountryDaily(&'a str),

    // Device keys
    InactiveDeviceObserver(&'a str),

    // Fetch consumer keys (chain, address)
    FetchCoinAddresses(&'a str, &'a str),
    FetchTokenAddresses(&'a str, &'a str),
    FetchNftAssetsAddresses(&'a str, &'a str),
    FetchAddressTransactions(&'a str, &'a str),

    // Asset keys
    FetchAssets(&'a str),
    Price(&'a str),
    PricerCoinInfo(&'a str),
    CoinInfoUpdate(&'a str),

    // Fiat keys
    FiatRates,
    FiatQuote(&'a str),
    FiatIpCheck(&'a str),

    // Auth keys (device_id, nonce)
    AuthNonce(&'a str, &'a str),

    // Status keys
    JobStatus(&'a str),
    ConsumerStatus(&'a str),
    ParserStatus(&'a str),

    // Pricer keys
    Markets,
    ObservedAssets,

    SwapDepositAddresses(&'a str),
    SwapSendAddresses(&'a str),

    // Alerter keys
    AlerterStakeRewards(&'a str, &'a str),

    // Perpetual keys
    PerpetualActiveAddresses(&'a str),
    PerpetualObserverCheckpoint(&'a str, &'a str),
}

pub fn cache_keys<'a, T: AsRef<str>>(items: &'a [T], variant: impl Fn(&'a str) -> CacheKey<'a>) -> Vec<String> {
    items.iter().map(|item| variant(item.as_ref()).key()).collect()
}

impl CacheKey<'_> {
    pub fn key(&self) -> String {
        match self {
            Self::ReferralIpCheck(ip_address) => format!("referral:ip_check:{}", ip_address),
            Self::UsernameCreationPerIp(ip_address) => format!("username:ip:{}", ip_address),
            Self::UsernameCreationPerDevice(device_id) => format!("username:device:{}", device_id),
            Self::UsernameCreationGlobalDaily => "username:global:daily".to_string(),
            Self::UsernameCreationPerCountryDaily(country) => format!("username:country:daily:{}", country),
            Self::InactiveDeviceObserver(device_id) => format!("device:inactive_observer:{}", device_id),
            Self::FetchCoinAddresses(chain, address) => format!("fetch:coin_addresses:{}:{}", chain, address),
            Self::FetchTokenAddresses(chain, address) => format!("fetch:token_addresses:{}:{}", chain, address),
            Self::FetchNftAssetsAddresses(chain, address) => format!("fetch:nft_assets_addresses:{}:{}", chain, address),
            Self::FetchAddressTransactions(chain, address) => format!("fetch:address_transactions:{}:{}", chain, address),
            Self::FetchAssets(asset_id) => format!("fetch:assets:{}", asset_id),
            Self::Price(asset_id) => format!("prices:{}", asset_id),
            Self::PricerCoinInfo(coin_id) => format!("pricer:coin_info:{}", coin_id),
            Self::CoinInfoUpdate(coin_id) => format!("coin_info:update:{}", coin_id),
            Self::FiatRates => "fiat:rates".to_string(),
            Self::FiatQuote(quote_id) => format!("fiat:quote:{}", quote_id),
            Self::FiatIpCheck(ip_address) => format!("fiat:ip_check:{}", ip_address),
            Self::AuthNonce(device_id, nonce) => format!("auth:nonce:{}:{}", device_id, nonce),
            Self::JobStatus(name) => format!("jobs:status:{}", name),
            Self::ConsumerStatus(name) => format!("consumers:status:{}", name),
            Self::ParserStatus(chain) => format!("parser:status:{}", chain),
            Self::Markets => "markets:markets".to_string(),
            Self::ObservedAssets => "pricer:observed_assets".to_string(),
            Self::SwapDepositAddresses(provider) => format!("swap:deposit_addresses:{}", provider),
            Self::SwapSendAddresses(provider) => format!("swap:send_addresses:{}", provider),
            Self::AlerterStakeRewards(chain, address) => format!("alerter:stake_rewards:{}:{}", chain, address),
            Self::PerpetualActiveAddresses(chain) => format!("perpetual:active_addresses:{}", chain),
            Self::PerpetualObserverCheckpoint(chain, address) => format!("perpetual:last_seen:{}:{}", chain, address),
        }
    }

    pub fn ttl(&self) -> u64 {
        match self {
            Self::ReferralIpCheck(_) => 30 * SECONDS_PER_DAY,
            Self::UsernameCreationPerIp(_) => 30 * SECONDS_PER_DAY,
            Self::UsernameCreationPerDevice(_) => 30 * SECONDS_PER_DAY,
            Self::UsernameCreationGlobalDaily => SECONDS_PER_DAY,
            Self::UsernameCreationPerCountryDaily(_) => SECONDS_PER_DAY,
            Self::InactiveDeviceObserver(_) => 30 * SECONDS_PER_DAY,
            Self::FetchCoinAddresses(_, _) => 7 * SECONDS_PER_DAY,
            Self::FetchTokenAddresses(_, _) => 30 * SECONDS_PER_DAY,
            Self::FetchNftAssetsAddresses(_, _) => 30 * SECONDS_PER_DAY,
            Self::FetchAddressTransactions(_, _) => 30 * SECONDS_PER_DAY,
            Self::FetchAssets(_) => 30 * SECONDS_PER_DAY,
            Self::Price(_) => 30 * SECONDS_PER_DAY,
            Self::PricerCoinInfo(_) => SECONDS_PER_DAY,
            Self::CoinInfoUpdate(_) => 90 * SECONDS_PER_DAY,
            Self::FiatRates => SECONDS_PER_DAY,
            Self::FiatQuote(_) => 15 * SECONDS_PER_MINUTE,
            Self::FiatIpCheck(_) => SECONDS_PER_DAY,
            Self::AuthNonce(_, _) => 5 * SECONDS_PER_MINUTE,
            Self::JobStatus(_) => 7 * SECONDS_PER_DAY,
            Self::ConsumerStatus(_) => 7 * SECONDS_PER_DAY,
            Self::ParserStatus(_) => 7 * SECONDS_PER_DAY,
            Self::Markets => SECONDS_PER_DAY,
            Self::ObservedAssets => 2 * SECONDS_PER_MINUTE,
            Self::SwapDepositAddresses(_) => SECONDS_PER_DAY,
            Self::SwapSendAddresses(_) => SECONDS_PER_DAY,
            Self::AlerterStakeRewards(_, _) => 30 * SECONDS_PER_DAY,
            Self::PerpetualActiveAddresses(_) => 2 * 60 * SECONDS_PER_MINUTE,
            Self::PerpetualObserverCheckpoint(_, _) => 30 * SECONDS_PER_DAY,
        }
    }
}
