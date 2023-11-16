use std::collections::HashSet;
use std::error::Error;
use std::time::Duration;

use crate::model::{FiatMappingMap, FiatMapping, FiatClient, FiatRates};
use crate::moonpay::MoonPayClient;
use crate::transak::TransakClient;
use crate::mercuryo::MercuryoClient;
use crate::ramp::RampClient;
use primitives::{fiat_quote::FiatQuote, fiat_quote_request::FiatBuyRequest, fiat_provider::FiatProviderName, fiat_assets::FiatAssets};
//use futures::future::join_all;
use storage::DatabaseClient;
use reqwest::Client as RequestClient;

pub struct Client {
    database: DatabaseClient,
    transak: TransakClient,
    moonpay: MoonPayClient,
    #[allow(dead_code)]
    mercuryo: MercuryoClient,
    #[allow(dead_code)]
    ramp: RampClient,
}

impl Client {
    pub async fn new(
        database_url: &str,
        transak: TransakClient, 
        moonpay: MoonPayClient,
        mercuryo: MercuryoClient,
        ramp: RampClient
    ) -> Self {
        let database = DatabaseClient::new(database_url);

        Self {
            database,
            transak,
            moonpay,
            mercuryo,
            ramp
        }
    }

    pub fn request_client(timeout_seconds: u64) -> RequestClient {
        RequestClient::builder()
            .timeout(Duration::from_secs(timeout_seconds))
            .build().unwrap()
    }

    pub async fn get_assets(&mut self) -> Result<FiatAssets, Box<dyn Error>> {
        let assets = self.database
            .get_fiat_assets()?
            .into_iter()
            .map(|x| x.asset_id)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        let version = self.database.get_fiat_assets_version()?;
        
        Ok(FiatAssets { version: version as u32, asset_ids: assets })
    }

    pub async fn get_fiat_rates(&mut self) -> Result<FiatRates, Box<dyn Error>> {
        let rates = self.database.get_fiat_rates()?;
        Ok(FiatRates { rates })
    }

    pub fn get_fiat_mapping(&mut self, asset_id: &str) -> Result<FiatMappingMap, Box<dyn Error>> {
        let list = self.database.get_fiat_assets_for_asset_id(asset_id)?;
        let mut map: FiatMappingMap = FiatMappingMap::new();
        list.into_iter().for_each(|x| {
            map.insert(x.provider, FiatMapping{symbol: x.symbol, network: x.network});
        });
        Ok(map)
        //Ok(map)
    }

    pub async fn get_quotes(&mut self, request: FiatBuyRequest, fiat_mapping_map: FiatMappingMap) -> Result<Vec<FiatQuote>, Box<dyn Error + Send + Sync>> {
        let mut results: Vec<FiatQuote> = vec![];
        
        //TODO: Implement fetching async quotes.
        if let Some(value) = fiat_mapping_map.get(self.moonpay.name().as_str()) {
            let result = self.moonpay.get_quote(request.clone(), value.clone()).await;
            match result {
                Ok(value) => { 
                    results.push(value)
                }
                Err(ee) => { 
                    println!("error {}", ee);
                }
            }
        }

        if let Some(value) = fiat_mapping_map.get(self.transak.name().as_str()) {
            let result = self.transak.get_quote(request.clone(), value.clone()).await;
            match result {
                Ok(value) => { 
                    results.push(value)
                }
                Err(ee) => { 
                    println!("error {}", ee);
                }
            }
        }

        if let Some(value) = fiat_mapping_map.get(self.mercuryo.name().as_str()) {
            let result = self.mercuryo.get_quote(request.clone(), value.clone()).await;
            match result {
                Ok(value) => { 
                    results.push(value)
                }
                Err(ee) => { 
                    println!("error {}", ee);
                }
            }
        }

        //let futures = vec![
            //self.moonpay.get_quote(request.clone(), fiat_mapping_map.get("MoonPay").expect("msg").clone()),
            //self.transak.get_quote(request.clone(), fiat_mapping_map.get("Transak").expect("msg").clone()),
            //self.get_quote(request.clone(), FiatProviderName::Transak, fiat_mapping_map.clone()),
            //self.get_quote(request.clone(), FiatProviderName::Mercuryo, fiat_mapping_map.clone()),
            //self.get_quote(request.clone(), FiatProviderName::MoonPay, fiat_mapping_map.clone()),
            //self.get_quote(request.clone(), FiatProviderName::Ramp, fiat_mapping_map.clone()),
        //];
        // let results: Vec<FiatQuote> = join_all(futures)
        //     .await
        //     .into_iter()
        //     .flatten()
        //     .map(|quote| {
        //         let mut result = quote.clone();
        //         result.crypto_amount = precision(quote.crypto_amount, 5);
        //         return result
        //     })
        //     .collect();

        Ok(results)
    }

    #[allow(dead_code)]
    async fn get_quote(&self, client: &dyn FiatClient, request: FiatBuyRequest, provider: FiatProviderName, fiat_mapping_map: FiatMappingMap) -> Result<FiatQuote, Box<dyn Error>> {
        let mapping = fiat_mapping_map.get(provider.as_str()).expect("no mapping for the asset");
        client.get_quote(request.clone(), mapping.clone()).await
    }
    
    //TODO: Refactor to simplify and later use async traits
    // async fn get_quote(&self, request: FiatBuyRequest, provider: FiatProviderName, fiat_mapping_map: FiatMappingMap) -> Result<FiatQuote, Box<dyn Error>> {
    //     let mapping = fiat_mapping_map.get(provider.as_str()).expect("no mapping for the asset");
    //     match provider {
    //         FiatProviderName::Mercuryo => {
    //             return self.mercuryo.get_quote(request.clone(), mapping.clone()).await;
    //         },
    //         FiatProviderName::Transak => {
    //             return self.transak.get_quote(request.clone(), mapping.clone()).await;
    //         },
    //         FiatProviderName::MoonPay => {
    //             return self.moonpay.get_quote(request.clone(), mapping.clone()).await;
    //         },
    //         FiatProviderName::Ramp => {
    //             return self.ramp.get_quote(request.clone(), mapping.clone()).await;
    //         },
    //         // FiatProviderName::Mercuryo => {
    //         //     let value = self.mercuryo
    //         //         .get_quote(request.clone(), mapping.clone())
    //         //         .await
    //         //         .expect("no mercuryo quote");
    //         //     Ok(value)
    //         // },
    //         // FiatProviderName::Transak => {
    //         //     let value = self.transak
    //         //         .get_quote(request.clone(), mapping.clone())
    //         //         .await
    //         //         .expect("no transak quote");
    //         //     Ok(value)
    //         // },
    //         // FiatProviderName::MoonPay => {
    //         //     let value = self.moonpay
    //         //         .get_quote(request.clone(), mapping.clone())
    //         //         .await
    //         //         .expect("no moonpay quote");
    //         //     Ok(value)
    //         // },
    //         // FiatProviderName::Ramp => {
    //         //     let value = self.ramp
    //         //         .get_quote(request.clone(), mapping.clone())
    //         //         .await
    //         //         .expect("no ramp quote");
    //         //     Ok(value)
    //         // },
    //     }
    // } 
}

#[allow(dead_code)]
fn precision(val: f64, precision: usize) -> f64 {
    format!("{:.prec$}", val, prec = precision).parse::<f64>().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_precision() {
        assert_eq!(precision(1.123, 2), 1.12);
        assert_eq!(precision(1.123, 5), 1.123);
    }
}