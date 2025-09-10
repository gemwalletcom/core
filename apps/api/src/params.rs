use primitives::Chain;
use rocket::form::{self, FromFormField, ValueField};
use rocket::request::FromParam;
use std::str::FromStr;

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
