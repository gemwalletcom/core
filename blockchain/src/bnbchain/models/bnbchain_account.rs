use typeshare::typeshare;

use super::bnbchain_balance::BNBChainBalance;

#[typeshare]
#[allow(dead_code)]
pub struct BNBChainAccount {
    pub balances: Vec<BNBChainBalance>,
    pub sequence: i32,
    pub account_number: i32,
}