pub mod client;
pub mod model;
pub mod provider;
pub use provider::FiatProvider;
pub mod providers;
use crate::providers::{BanxaClient, MercuryoClient, MoonPayClient, RampClient, TransakClient};
use settings::Settings;

pub struct FiatProviderFactory {}
impl FiatProviderFactory {
    pub fn new_providers(settings: Settings) -> Vec<Box<dyn FiatProvider + Send + Sync>> {
        let request_client = crate::client::Client::request_client(settings.fiat.timeout);

        let moonpay = MoonPayClient::new(
            request_client.clone(),
            settings.moonpay.key.public.clone(),
            settings.moonpay.key.secret.clone(),
        );
        let ramp = RampClient::new(request_client.clone(), settings.ramp.key.public.clone());
        let mercuryo = MercuryoClient::new(
            request_client.clone(),
            settings.mercuryo.key.public.clone(),
            settings.mercuryo.key.secret.clone(),
        );
        let transak = TransakClient::new(request_client.clone(), settings.transak.key.public);
        let banxa = BanxaClient::new(
            request_client.clone(),
            settings.banxa.url,
            settings.banxa.key.public,
            settings.banxa.key.secret,
        );

        vec![
            Box::new(moonpay),
            Box::new(ramp),
            Box::new(mercuryo),
            Box::new(transak),
            Box::new(banxa),
        ]
    }
}
