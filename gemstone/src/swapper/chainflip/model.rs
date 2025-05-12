use serde::{Deserialize, Serialize};

use super::broker::DcaParameters;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ChainflipRouteData {
    pub boost_fee: Option<u32>,
    pub fee_bps: u32,
    pub estimated_price: String,
    pub dca_params: Option<DcaParameters>,
}
