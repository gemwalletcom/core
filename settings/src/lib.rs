use config::{Config, ConfigError, File, Environment};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Settings {
    pub redis: Database,
    pub postgres: Database,
    pub fiat: Fiat,
    pub moonpay: MoonPay,
    pub transak: Transak,
    pub mercuryo: Mercuryo,
    pub ramp: Mercuryo,
    pub coingecko: CoinGecko,
    pub pricer: Pricer,
    pub plausible: Plausible,
    pub name: Name,
    pub metrics: Metrics,
    pub assets: Assets,
    pub chains: Chains,
    pub pusher: Pusher,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Fiat {
    pub timeout: u64,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Database {
    pub url: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct KeyPublic {
    pub public: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct KeySecret {
    pub secret: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Key {
    pub secret: String,
    pub public: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct MoonPay {
    pub key: Key,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Transak {
    pub key: KeyPublic,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Mercuryo {
    pub key: KeyPublic,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Ramp {
    pub key: KeyPublic,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct CoinGecko {
    pub key: KeySecret,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Pricer {
    pub timer: u64,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Plausible {
    pub url: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Name {
    pub ens: ENS,
    pub ud: UD,
    pub sns: SNS,
    pub ton: TON,
    pub tree: TREE,
    pub spaceid: SpaceId,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct ENS {
    pub url: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct SNS {
    pub url: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct TON {
    pub url: String,
}


#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct UD {
    pub url: String,
    pub key: KeySecret,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct TREE {
    pub url: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct SpaceId {
    pub url: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Metrics {
    pub path: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Assets {
    pub url: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Chains {
    pub binance: BNBChain,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Chain {
    pub url: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct BNBChain {
    pub url: String,
    pub api: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Pusher {
    pub url: String,
    pub ios: PusherIOS,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct PusherIOS {
    pub topic: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name("./Settings"))
            .add_source(Environment::with_prefix("").prefix_separator("").separator("_"))
            .build()?;
        s.try_deserialize()
    }
}
