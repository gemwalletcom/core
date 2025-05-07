use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChainflipRouteData {
    pub boost_fee: Option<u32>,
    pub fee_bps: u32,
}
