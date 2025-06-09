use primitives::{Chain, Transaction};

use crate::rpc::alchemy::model::Transfer;

pub struct AlchemyMapper {}

impl AlchemyMapper {
    pub fn map_transaction(_transfer: Transfer, _chain: Chain) -> Option<Transaction> {
        None
    }
}
