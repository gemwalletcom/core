#![allow(unused)]

mod duration;

use serde::Deserialize;
use std::{env, path::PathBuf, time::Duration};

use config::{Config, ConfigError, Environment, File};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub redis: Redis,
    pub postgres: Postgres,
    pub meilisearch: MeiliSearch,
    pub rabbitmq: RabbitMQ,

    pub api: API,
    pub parser: Parser,
    pub daemon: Daemon,
    pub consumer: Consumer,

    pub fiat: Fiat,
    pub moonpay: MoonPay,
    pub transak: Transak,
    pub mercuryo: Mercuryo,
    pub banxa: Banxa,
    pub paybis: Paybis,

    pub prices: Prices,
    pub coingecko: CoinGecko,
    pub charter: Charter,
    pub name: Name,
    pub metrics: Metrics,
    pub chains: Chains,
    pub pusher: Pusher,
    pub scan: Scan,
    pub nft: NFT,
    pub ankr: Ankr,
    pub trongrid: Trongrid,
    pub assets: Assets,
    pub sentry: Option<Sentry>,
    pub rewards: Rewards,
    pub ip: Ip,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Fiat {
    #[serde(deserialize_with = "duration::deserialize")]
    pub timeout: Duration,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Redis {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Postgres {
    pub url: String,
    pub pool: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Retry {
    #[serde(deserialize_with = "duration::deserialize")]
    pub delay: Duration,
    #[serde(deserialize_with = "duration::deserialize")]
    pub timeout: Duration,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RabbitMQ {
    pub url: String,
    pub prefetch: u16,
    pub retry: Retry,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MeiliSearch {
    pub url: String,
    pub key: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct KeyPublic {
    pub public: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct KeySecret {
    pub secret: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Key {
    pub secret: String,
    pub public: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct KeySettings {
    pub key: Key,
}
pub type MoonPay = KeySettings;
pub type Paybis = KeySettings;

#[derive(Debug, Deserialize, Clone)]
pub struct Transak {
    pub key: Key,
    pub referrer_domain: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Mercuryo {
    pub key: MercuryoKey,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MercuryoKey {
    pub secret: String,
    pub public: String,
    pub token: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SecretKeySettings {
    pub key: KeySecret,
}
pub type CoinGecko = SecretKeySettings;

#[derive(Debug, Deserialize, Clone)]
pub struct UrlSecretKeySettings {
    pub url: String,
    pub key: KeySecret,
}
pub type UD = UrlSecretKeySettings;
pub type Banxa = ScanProvider;

#[derive(Debug, Deserialize, Clone)]
pub struct Prices {
    pub pyth: PriceProvider,
    pub jupiter: PriceProvider,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PriceProvider {
    pub url: String,
    pub timer: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Charter {
    pub timer: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Name {
    pub ens: URL,
    pub ud: UD,
    pub sns: URL,
    pub ton: URL,
    pub eths: URL,
    pub spaceid: URL,
    pub did: URL,
    pub suins: URL,
    pub aptos: URL,
    pub injective: URL,
    pub icns: URL,
    pub lens: URL,
    pub base: URL,
    pub hyperliquid: URL,
    pub alldomains: URL,
}

#[derive(Debug, Deserialize, Clone)]
pub struct URL {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Metrics {
    pub path: String,
    pub redis: Redis,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Chains {
    pub solana: Chain,
    pub ethereum: Chain,
    pub smartchain: Chain,
    pub polygon: Chain,
    pub optimism: Chain,
    pub arbitrum: Chain,
    pub base: Chain,
    pub opbnb: Chain,
    pub avalanchec: Chain,
    pub ton: Chain,
    pub cosmos: Chain,
    pub osmosis: Chain,
    pub thorchain: Chain,
    pub celestia: Chain,
    pub tron: Chain,
    pub xrp: Chain,
    pub aptos: Chain,
    pub sui: Chain,
    pub bitcoin: Chain,
    pub bitcoincash: Chain,
    pub litecoin: Chain,
    pub doge: Chain,
    pub zcash: Chain,
    pub fantom: Chain,
    pub gnosis: Chain,
    pub injective: Chain,
    pub sei: Chain,
    pub manta: Chain,
    pub blast: Chain,
    pub noble: Chain,
    pub zksync: Chain,
    pub linea: Chain,
    pub mantle: Chain,
    pub celo: Chain,
    pub near: Chain,
    pub world: Chain,
    pub plasma: Chain,
    pub stellar: Chain,
    pub sonic: Chain,
    pub algorand: Chain,
    pub polkadot: Chain,
    pub cardano: Chain,
    #[serde(rename = "abstract")]
    pub abstract_chain: Chain,
    pub berachain: Chain,
    pub ink: Chain,
    pub unichain: Chain,
    pub hyperliquid: Chain,
    pub hypercore: Chain,
    pub monad: Chain,
    pub xlayer: Chain,
    pub stable: Chain,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Chain {
    pub url: String,
    #[serde(default)]
    pub node: ChainURLType,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub enum ChainURLType {
    #[default]
    Default,
    Archival,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Shutdown {
    #[serde(deserialize_with = "duration::deserialize")]
    pub timeout: Duration,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Parser {
    #[serde(deserialize_with = "duration::deserialize")]
    pub timeout: Duration,
    pub shutdown: Shutdown,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Daemon {
    pub service: String,
    pub shutdown: Shutdown,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Consumer {
    pub error: ConsumerError,
    #[serde(default, deserialize_with = "duration::deserialize")]
    pub delay: Duration,
    pub shutdown: Shutdown,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ConsumerError {
    #[serde(deserialize_with = "duration::deserialize")]
    pub timeout: Duration,
    pub skip: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct API {
    pub service: String,
    pub auth: Auth,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Auth {
    pub enabled: bool,
    #[serde(deserialize_with = "duration::deserialize")]
    pub tolerance: Duration,
    pub jwt: Jwt,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Jwt {
    pub secret: String,
    #[serde(deserialize_with = "duration::deserialize")]
    pub expiry: Duration,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Pusher {
    pub url: String,
    pub ios: PusherIOS,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PusherIOS {
    pub topic: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Scan {
    #[serde(deserialize_with = "duration::deserialize")]
    pub timeout: Duration,
    pub hashdit: ScanProvider,
    pub goplus: ScanProvider,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ScanProvider {
    pub url: String,
    pub key: Key,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let current_dir = env::current_dir().unwrap();
        Self::new_setting_path(current_dir.join("Settings.yaml"))
    }

    pub fn new_setting_path(path: PathBuf) -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::from(path))
            .add_source(Environment::with_prefix("").prefix_separator("").separator("_"))
            .build()?;
        s.try_deserialize()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct NFT {
    pub nftscan: NFTScan,
    pub opensea: OpenSea,
    pub magiceden: MagicEden,
    pub bucket: BucketConfiguration,
}
pub type Ankr = SecretKeySettings;
pub type Trongrid = SecretKeySettings;
pub type NFTScan = SecretKeySettings;
pub type OpenSea = SecretKeySettings;
pub type MagicEden = SecretKeySettings;

#[derive(Debug, Deserialize, Clone)]
pub struct BucketConfiguration {
    pub endpoint: String,
    pub region: String,
    pub key: Key,
    pub name: String,
    pub url: String,
}

pub type Assets = URL;

#[derive(Debug, Deserialize, Clone)]
pub struct Sentry {
    pub dsn: String,
    pub sample_rate: f32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Rewards {
    #[serde(default)]
    pub wallets: HashMap<String, RewardsWallet>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RewardsWallet {
    pub key: String,
    pub address: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Ip {
    pub abuseipdb: AbuseIPDB,
    pub ipapi: IpApi,
}
pub type AbuseIPDB = UrlSecretKeySettings;
pub type IpApi = UrlSecretKeySettings;

#[cfg(feature = "testkit")]
pub mod testkit;

pub fn service_user_agent(service: &str, sub_service: Option<&str>) -> String {
    match sub_service {
        Some(sub) => format!("{}_{}", service, sub),
        None => service.to_string(),
    }
}
