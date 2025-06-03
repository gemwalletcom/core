use client::NameClient;
use settings::Settings;

pub mod aptos;
pub mod base;
pub mod client;
pub mod codec;
pub mod did;
pub mod ens;
pub mod eths;
pub mod hyperliquid;
pub mod icns;
pub mod injective;
pub mod lens;
pub mod sns;
pub mod spaceid;
pub mod suins;
pub mod ton;
pub mod ton_codec;
pub mod ud;
pub struct NameProviderFactory {}

impl NameProviderFactory {
    pub fn create_providers(settings: Settings) -> Vec<Box<dyn NameClient + Send + Sync>> {
        vec![
            Box::new(ens::ENSClient::new(settings.name.ens.url)),
            Box::new(ud::UDClient::new(settings.name.ud.url, settings.name.ud.key.secret)),
            Box::new(sns::SNSClient::new(settings.name.sns.url)),
            Box::new(ton::TONClient::new(settings.name.ton.url)),
            Box::new(eths::EthsClient::new(settings.name.eths.url)),
            Box::new(spaceid::SpaceIdClient::new(settings.name.spaceid.url)),
            Box::new(did::DidClient::new(settings.name.did.url)),
            Box::new(suins::SuinsClient::new(settings.name.suins.url)),
            Box::new(aptos::AptosClient::new(settings.name.aptos.url)),
            Box::new(injective::InjectiveNameClient::new(settings.name.injective.url)),
            Box::new(icns::IcnsClient::new(settings.name.icns.url)),
            Box::new(lens::LensClient::new(settings.name.lens.url)),
            Box::new(base::Basenames::new(settings.name.base.url)),
            Box::new(hyperliquid::Hyperliquid::new(settings.name.hyperliquid.url)),
        ]
    }
}
