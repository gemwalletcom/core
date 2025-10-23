use primitives::Chain;

pub(crate) fn deposit_memo_chains() -> &'static [Chain] {
    &[Chain::Stellar]
}

pub(crate) fn auto_quote_time_chains() -> &'static [Chain] {
    &[Chain::Gnosis]
}
