use primitives::currency::Currency;
use primitives::{AssetId, Chain, ChartPeriod, Device, FiatProviderName, FiatQuoteType, NFTAssetId, NFTCollectionId, SwapProvider, TransactionId};
use rocket::data::{FromData, Outcome, ToByteUnit};
use rocket::form::{self, FromFormField, ValueField};
use rocket::http::Status;
use rocket::outcome::Outcome::{Error, Success};
use rocket::request::FromParam;
use rocket::{Data, Request};
use std::str::FromStr;
use unic_langid::LanguageIdentifier;

const MAX_ADDRESS_LENGTH: usize = 256;
const MAX_ASSET_ID_LENGTH: usize = 256;
const MAX_SEARCH_QUERY_LENGTH: usize = 128;

pub struct ChainParam(pub Chain);

impl<'r> FromParam<'r> for ChainParam {
    type Error = &'r str;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        Chain::from_str(param).map(ChainParam).map_err(|_| param)
    }
}

impl<'r> FromFormField<'r> for ChainParam {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        Chain::from_str(field.value)
            .map(ChainParam)
            .map_err(|_| form::Error::validation(format!("Invalid chain: {}", field.value)).into())
    }
}

pub struct TransactionIdParam(pub TransactionId);

impl<'r> FromParam<'r> for TransactionIdParam {
    type Error = &'r str;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        TransactionId::from_str(param).map(TransactionIdParam).map_err(|_| param)
    }
}

pub struct FiatQuoteTypeParam(pub FiatQuoteType);

impl<'r> FromParam<'r> for FiatQuoteTypeParam {
    type Error = &'r str;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        FiatQuoteType::from_str(param).map(FiatQuoteTypeParam).map_err(|_| param)
    }
}

pub struct CurrencyParam(pub Currency);

impl<'r> FromFormField<'r> for CurrencyParam {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        Currency::from_str(field.value).map(CurrencyParam).or_else(|_| Ok(CurrencyParam(Currency::USD)))
    }
}

impl CurrencyParam {
    pub fn as_string(&self) -> String {
        self.0.as_ref().to_string()
    }
}

pub struct AddressParam(pub String);

impl<'r> FromParam<'r> for AddressParam {
    type Error = &'r str;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        if param.is_empty() || param.len() > MAX_ADDRESS_LENGTH {
            return Err(param);
        }
        Ok(AddressParam(param.to_string()))
    }
}

impl<'r> FromFormField<'r> for AddressParam {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        if field.value.is_empty() || field.value.len() > MAX_ADDRESS_LENGTH {
            return Err(form::Error::validation(format!("Invalid address: {}", field.value)).into());
        }
        Ok(AddressParam(field.value.to_string()))
    }
}

pub struct NftCollectionIdParam(pub NFTCollectionId);

impl<'r> FromParam<'r> for NftCollectionIdParam {
    type Error = &'r str;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        NFTCollectionId::from_id(param).map(NftCollectionIdParam).ok_or(param)
    }
}

pub struct NftAssetIdParam(pub NFTAssetId);

impl<'r> FromParam<'r> for NftAssetIdParam {
    type Error = &'r str;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        NFTAssetId::from_id(param).map(NftAssetIdParam).ok_or(param)
    }
}

pub struct AssetIdParam(pub AssetId);

impl<'r> FromParam<'r> for AssetIdParam {
    type Error = &'r str;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        if param.is_empty() || param.len() > MAX_ASSET_ID_LENGTH {
            return Err(param);
        }
        AssetId::new(param).map(AssetIdParam).ok_or(param)
    }
}

impl<'r> FromFormField<'r> for AssetIdParam {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        if field.value.is_empty() || field.value.len() > MAX_ASSET_ID_LENGTH {
            return Err(form::Error::validation(format!("Invalid asset_id: {}", field.value)).into());
        }
        AssetId::new(field.value)
            .map(AssetIdParam)
            .ok_or_else(|| form::Error::validation(format!("Invalid asset_id: {}", field.value)).into())
    }
}

pub struct ChartPeriodParam(pub ChartPeriod);

impl<'r> FromParam<'r> for ChartPeriodParam {
    type Error = &'r str;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        ChartPeriod::new(param.to_string()).map(ChartPeriodParam).ok_or(param)
    }
}

impl<'r> FromFormField<'r> for ChartPeriodParam {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        ChartPeriod::new(field.value.to_string())
            .map(ChartPeriodParam)
            .ok_or_else(|| form::Error::validation(format!("Invalid period: {}", field.value)).into())
    }
}

pub struct SwapProviderParam(pub SwapProvider);

impl<'r> FromParam<'r> for SwapProviderParam {
    type Error = &'r str;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        SwapProvider::from_str(param).map(SwapProviderParam).map_err(|_| param)
    }
}

pub struct FiatProviderIdParam(pub FiatProviderName);

impl<'r> FromFormField<'r> for FiatProviderIdParam {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        FiatProviderName::from_str(field.value)
            .map(FiatProviderIdParam)
            .map_err(|_| form::Error::validation(format!("Invalid provider: {}", field.value)).into())
    }
}

pub struct SearchQueryParam(pub String);

impl<'r> FromFormField<'r> for SearchQueryParam {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        if field.value.len() > MAX_SEARCH_QUERY_LENGTH {
            return Err(form::Error::validation(format!("Invalid query length: {}", field.value.len())).into());
        }
        Ok(SearchQueryParam(field.value.to_string()))
    }
}

pub struct UserAgent(pub String);

#[rocket::async_trait]
impl<'r> rocket::request::FromRequest<'r> for UserAgent {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        match request.headers().get_one(rocket::http::hyper::header::USER_AGENT.as_str()) {
            Some(ua) => rocket::request::Outcome::Success(UserAgent(ua.to_string())),
            None => rocket::request::Outcome::Error((Status::BadRequest, ())),
        }
    }
}

pub struct DeviceParam(pub Device);

#[rocket::async_trait]
impl<'r> FromData<'r> for DeviceParam {
    type Error = String;

    async fn from_data(_req: &'r Request<'_>, data: Data<'r>) -> Outcome<'r, Self> {
        let Ok(bytes) = data.open(64.kibibytes()).into_bytes().await else {
            return Error((Status::BadRequest, "Failed to read body".to_string()));
        };
        if !bytes.is_complete() {
            return Error((Status::BadRequest, "Request body too large".to_string()));
        }

        let Ok(device) = serde_json::from_slice::<Device>(&bytes.into_inner()) else {
            return Error((Status::BadRequest, "Invalid JSON".to_string()));
        };

        if device.locale.parse::<LanguageIdentifier>().is_err() {
            return Error((Status::BadRequest, format!("Invalid locale: {}", device.locale)));
        }

        Success(DeviceParam(device))
    }
}
