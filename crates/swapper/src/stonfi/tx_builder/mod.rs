mod message;
mod model;
#[cfg(test)]
mod tests;
mod v1;
mod v2;

use super::model::Router;
use crate::SwapperError;

pub use model::{ReferralParams, SwapTransactionParams, TxParams};

#[derive(Debug, Clone, Copy)]
enum RouterVersion {
    V1,
    V2,
}

pub fn build_swap_transaction(params: SwapTransactionParams<'_>) -> Result<TxParams, SwapperError> {
    match router_version(&params.simulation.router)? {
        RouterVersion::V1 => v1::build_swap_transaction(params),
        RouterVersion::V2 => v2::build_swap_transaction(params),
    }
}

fn router_version(router: &Router) -> Result<RouterVersion, SwapperError> {
    match router.major_version {
        1 => Ok(RouterVersion::V1),
        2 => match router.minor_version {
            1 | 2 => Ok(RouterVersion::V2),
            minor => Err(SwapperError::ComputeQuoteError(format!("Unsupported STON.fi v2 router minor version: {minor}"))),
        },
        major => Err(SwapperError::ComputeQuoteError(format!("Unsupported STON.fi router major version: {major}"))),
    }
}
