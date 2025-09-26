use serde::Deserialize;
use std::{env, path::PathBuf};

use config::{Config, ConfigError, Environment, File};

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Settings {
    pub redis: Database,
    pub postgres: Database,
    pub meilisearch: MeiliSearch,
    pub rabbitmq: Database,

    pub api: API,
    pub parser: Parser,
    pub daemon: Daemon,

    pub fiat: Fiat,
    pub moonpay: MoonPay,
    pub transak: Transak,
    pub mercuryo: Mercuryo,
    pub banxa: Banxa,
    pub paybis: Paybis,

    pub pricer: Pricer,
    pub coingecko: CoinGecko,
    pub charter: Charter,
    pub name: Name,
    pub metrics: Metrics,
    pub chains: Chains,
    pub pusher: Pusher,
    pub alerter: Alerter,
    pub scan: Scan,
    pub nft: NFT,
    pub ankr: Ankr,
    pub trongrid: Trongrid,
    pub assets: Assets,
    pub sentry: Option<Sentry>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Fiat {
    pub timeout: u64,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Database {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct MeiliSearch {
    pub url: String,
    pub key: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct KeyPublic {
    pub public: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct KeySecret {
    pub secret: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Key {
    pub secret: String,
    pub public: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct MoonPay {
    pub key: Key,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Transak {
    pub key: Key,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Mercuryo {
    pub key: MercuryoKey,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct MercuryoKey {
    pub secret: String,
    pub public: String,
    pub token: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Banxa {
    pub url: String,
    pub key: Key,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Paybis {
    pub key: Key,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct CoinGecko {
    pub key: KeySecret,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Pricer {
    pub timer: u64,
    pub outdated: u64,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Charter {
    pub timer: u64,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
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
#[allow(unused)]
pub struct URL {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct UD {
    pub url: String,
    pub key: KeySecret,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Metrics {
    pub path: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
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
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Chain {
    pub url: String,
    pub archive_url: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub enum ChainURLType {
    Default(String),
    Archive(String),
}

impl ChainURLType {
    pub fn get_url(&self) -> Option<String> {
        match self {
            ChainURLType::Default(url) => Some(url.clone()),
            ChainURLType::Archive(_) => None,
        }
    }

    pub fn get_archive_url(&self) -> Option<String> {
        match self {
            ChainURLType::Default(_) => None,
            ChainURLType::Archive(url) => Some(url.clone()),
        }
    }
}

impl Chain {
    pub fn get_type(&self) -> (ChainURLType, Option<ChainURLType>) {
        if let Some(url) = self.archive_url.clone() {
            (ChainURLType::Default(self.url.clone()), Some(ChainURLType::Archive(url)))
        } else {
            (ChainURLType::Default(self.url.clone()), None)
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Parser {
    pub timeout: u64,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Daemon {
    pub service: String,
    pub search: DaemonSearch,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct API {
    pub service: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Pusher {
    pub url: String,
    pub ios: PusherIOS,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct PusherIOS {
    pub topic: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Scan {
    pub timeout_ms: u64,
    pub hashdit: ScanProvider,
    pub goplus: ScanProvider,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct ScanProvider {
    pub url: String,
    pub key: Key,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Alerter {
    pub update_interval_seconds: u64,
    pub rules: AlerterRules,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct AlerterRules {
    pub price_increase_percent: f64,
    pub price_decrease_percent: f64,
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
#[allow(unused)]
pub struct NFT {
    pub nftscan: NFTScan,
    pub opensea: OpenSea,
    pub magiceden: MagicEden,
    pub bucket: BucketConfiguration,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Ankr {
    pub key: KeySecret,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Trongrid {
    pub key: KeySecret,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct NFTScan {
    pub key: KeySecret,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct OpenSea {
    pub key: KeySecret,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct MagicEden {
    pub key: KeySecret,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct BucketConfiguration {
    pub endpoint: String,
    pub region: String,
    pub key: Key,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct DaemonSearch {
    pub assets_update_interval: u64,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Assets {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Sentry {
    pub dsn: String,
    pub sample_rate: f32,
}

#[cfg(feature = "testkit")]
pub mod testkit;
