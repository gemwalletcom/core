#[typeshare]
struct BNBChainAccount {
    balances: Vec<BNBChainBalance>,
    sequence: Int,
    account_number: Int,
}