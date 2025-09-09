use primitives::Chain;
use rocket::request::FromParam;
use std::str::FromStr;

pub struct ChainParam(pub Chain);

impl<'r> FromParam<'r> for ChainParam {
    type Error = &'r str;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        Chain::from_str(param)
            .map(ChainParam)
            .map_err(|_| param)
    }
}