use primitives::currency::Currency;
use primitives::{Chain, ChartPeriod, FiatQuoteType, TransactionId};
use rocket::form::{self, FromFormField, ValueField};
use rocket::request::FromParam;
use std::str::FromStr;

const MAX_ADDRESS_LENGTH: usize = 256;
const MAX_ASSET_ID_LENGTH: usize = 128;
const MAX_DEVICE_ID_LENGTH: usize = 32;
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

pub struct AssetIdParam(pub String);

impl<'r> FromParam<'r> for AssetIdParam {
    type Error = &'r str;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        if param.is_empty() || param.len() > MAX_ASSET_ID_LENGTH {
            return Err(param);
        }
        Ok(AssetIdParam(param.to_string()))
    }
}

pub struct DeviceIdParam(pub String);

impl<'r> FromParam<'r> for DeviceIdParam {
    type Error = &'r str;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        if param.is_empty() || param.len() > MAX_DEVICE_ID_LENGTH {
            return Err(param);
        }
        Ok(DeviceIdParam(param.to_string()))
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

pub struct SearchQueryParam(pub String);

impl<'r> FromFormField<'r> for SearchQueryParam {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        if field.value.len() > MAX_SEARCH_QUERY_LENGTH {
            return Err(form::Error::validation(format!("Invalid query length: {}", field.value.len())).into());
        }
        Ok(SearchQueryParam(field.value.to_string()))
    }
}
