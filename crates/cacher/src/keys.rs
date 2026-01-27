const SECONDS_PER_MINUTE: u64 = 60;
const SECONDS_PER_DAY: u64 = 24 * 60 * 60;

pub enum CacheKey<'a> {
    ParserCurrentBlock(&'a str),
    ParserLatestBlock(&'a str),

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
            Self::ParserCurrentBlock(chain) => format!("parser:state:{}:current_block", chain),
            Self::ParserLatestBlock(chain) => format!("parser:state:{}:latest_block", chain),
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
            Self::PricerCoinInfo(coin_id) => format!("pricer:coin_info:{}", coin_id),
            Self::FiatQuote(quote_id) => format!("fiat:quote:{}", quote_id),
            Self::FiatIpCheck(ip_address) => format!("fiat:ip_check:{}", ip_address),
            Self::AuthNonce(device_id, nonce) => format!("auth:nonce:{}:{}", device_id, nonce),
        }
    }

    pub fn ttl(&self) -> u64 {
        match self {
            Self::ParserCurrentBlock(_) => 7 * SECONDS_PER_DAY,
            Self::ParserLatestBlock(_) => 7 * SECONDS_PER_DAY,
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
            Self::PricerCoinInfo(_) => SECONDS_PER_DAY,
            Self::FiatQuote(_) => 15 * SECONDS_PER_MINUTE,
            Self::FiatIpCheck(_) => SECONDS_PER_DAY,
            Self::AuthNonce(_, _) => 5 * SECONDS_PER_MINUTE,
        }
    }
}
