use crate::{
    Quote, SwapperError,
    mayan::{
        constants::HC_HYPEREVM_DEPOSIT_PROCESSOR,
        model::{MayanQuoteCommon, MayanSwiftQuote},
        wormhole_chain::{self, WormholeChain},
    },
};
use std::borrow::Cow;

pub(super) fn quote_destination_address(quote: &Quote) -> &str {
    if quote.request.destination_address.is_empty() {
        quote.request.wallet_address.as_str()
    } else {
        quote.request.destination_address.as_str()
    }
}

pub(super) fn swift_destination_address<'a>(quote: &'a Quote, route: &MayanSwiftQuote) -> Cow<'a, str> {
    if is_hypercore_deposit(route) {
        Cow::Borrowed(HC_HYPEREVM_DEPOSIT_PROCESSOR)
    } else {
        Cow::Borrowed(quote_destination_address(quote))
    }
}

pub(super) fn swift_destination_chain(route: &MayanSwiftQuote) -> &str {
    if is_hypercore_deposit(route) { WormholeChain::Hyperevm.name() } else { &route.to_chain }
}

pub(super) fn swift_destination_chain_id(route: &MayanSwiftQuote) -> Result<u16, SwapperError> {
    wormhole_chain::id_for_name(swift_destination_chain(route))
}

pub(super) fn is_hypercore_deposit(route: &MayanQuoteCommon) -> bool {
    route.to_chain == WormholeChain::Hypercore.name()
}
