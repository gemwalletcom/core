pub mod client;
pub mod model;
pub mod provider;
pub use provider::FiatProvider;
pub mod hmac_signature;
pub mod ip_check_client;
pub mod providers;

use crate::providers::{BanxaClient, MercuryoClient, MoonPayClient, PaybisClient, TransakClient};
use settings::Settings;
use std::time::Duration;
pub mod error;

pub use client::FiatClient;
pub use ip_check_client::{IPAddressInfo, IPCheckClient};

#[derive(Debug, Clone)]
pub struct FiatConfig {
    pub timeout: Duration,
    pub validate_subscription: bool,
}

impl FiatConfig {
    pub fn new(timeout: Duration, validate_subscription: bool) -> Self {
        Self {
            timeout,
            validate_subscription,
        }
    }
}

#[cfg(all(test, feature = "fiat_integration_tests"))]
pub mod testkit;

pub struct FiatProviderFactory {}
impl FiatProviderFactory {
    pub fn new_providers(settings: Settings) -> Vec<Box<dyn FiatProvider + Send + Sync>> {
        let request_client = crate::client::FiatClient::request_client(settings.fiat.timeout);

        let moonpay = MoonPayClient::new(request_client.clone(), settings.moonpay.key.public.clone(), settings.moonpay.key.secret.clone());
        let mercuryo = MercuryoClient::new(
            request_client.clone(),
            settings.mercuryo.key.public.clone(),
            settings.mercuryo.key.secret.clone(),
            settings.mercuryo.key.token.clone(),
        );
        let transak = TransakClient::new(request_client.clone(), settings.transak.key.public, settings.transak.key.secret);
        let banxa = BanxaClient::new(request_client.clone(), settings.banxa.url, settings.banxa.key.public, settings.banxa.key.secret);
        let paybis = PaybisClient::new(request_client.clone(), settings.paybis.key.public, settings.paybis.key.secret);

        vec![Box::new(moonpay), Box::new(mercuryo), Box::new(transak), Box::new(banxa), Box::new(paybis)]
    }

    pub fn new_ip_check_client(settings: Settings) -> IPCheckClient {
        let request_client = crate::client::FiatClient::request_client(settings.fiat.timeout);
        let moonpay = MoonPayClient::new(request_client.clone(), settings.moonpay.key.public.clone(), settings.moonpay.key.secret.clone());
        IPCheckClient::new(moonpay)
    }
}
