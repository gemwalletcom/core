mod contracts;
mod order;
mod transaction;

use crate::mayan::{client::MayanClient, model::MayanSwiftQuote};
use crate::{Quote, SwapperError, SwapperQuoteData, mayan::tx_builder::evm as evm_builder};
use gem_client::Client;
use std::{fmt::Debug, sync::Arc};

use crate::alien::RpcProvider;

pub async fn build_quote_data<C>(client: &MayanClient<C>, quote: &Quote, route: &MayanSwiftQuote, rpc_provider: Arc<dyn RpcProvider>) -> Result<SwapperQuoteData, SwapperError>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    evm_builder::build_quote_data(transaction::build(client, quote, route), quote, rpc_provider).await
}

#[cfg(test)]
mod tests {
    use super::order::swift_order;
    use super::*;
    use crate::mayan::{
        constants::HYPERCORE_SPOT_USDC_CONTRACT,
        model::{HcSwiftDeposit, MayanQuote},
        tx_builder::{address::native_address_to_bytes32, hypercore::hypercore_custom_payload, route::quote_destination_address},
        wormhole_chain::WormholeChain,
    };
    use gem_evm::EVM_ZERO_ADDRESS;
    use gem_solana::SYSTEM_PROGRAM_ID;
    use primitives::Chain;
    use primitives::decode_hex;

    fn swift_route() -> MayanSwiftQuote {
        let route: MayanQuote = serde_json::from_str(include_str!("../../test/swift_quote_evm_to_solana.json")).unwrap();
        route.as_swift().unwrap().clone()
    }

    #[test]
    fn test_native_address_to_bytes32() {
        assert_eq!(native_address_to_bytes32(SYSTEM_PROGRAM_ID, 1).unwrap(), [0u8; 32]);
        assert_eq!(native_address_to_bytes32(EVM_ZERO_ADDRESS, 2).unwrap(), [0u8; 32]);
    }

    #[test]
    fn test_swift_random_key_with_memo() {
        let random = super::super::swift_random_key(&swift_route()).unwrap();
        let mut expected = [0u8; 32];
        expected[15] = 1;
        expected[31] = 2;
        assert_eq!(random, expected);
    }

    #[test]
    fn test_swift_order() {
        let mut quote = crate::Quote::mock(Chain::Ethereum, None);
        quote.request.wallet_address = "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string();
        quote.request.destination_address = "7g2rVN8fAAQdPh1mkajpvELqYa3gWvFXJsBLnKfEQfqy".to_string();
        let order = swift_order(&quote, &swift_route(), 2, quote_destination_address(&quote), None).unwrap();
        assert_eq!(order.destChainId, 1);
        assert_eq!(order.minAmountOut, 118_381_871);
        assert_eq!(order.referrerBps, 50);
        assert_eq!(order.auctionMode, 2);
    }

    #[test]
    fn test_hypercore_custom_payload() {
        let mut route = swift_route();
        route.common.to_chain = WormholeChain::Hypercore.name().to_string();
        route.common.to_token.contract = HYPERCORE_SPOT_USDC_CONTRACT.to_string();
        route.common.to_token.w_chain_id = 65000;
        route.common.to_token.decimals = 6;
        route.hc_swift_deposit = Some(HcSwiftDeposit {
            relayer_fee64: "500000".to_string(),
        });

        let destination = "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7";
        let payload = hypercore_custom_payload(&route, destination).unwrap().unwrap();

        assert_eq!(payload.len(), 32);
        assert_eq!(&payload[..20], decode_hex(destination).unwrap().as_slice());
        assert_eq!(&payload[20..24], 65535u32.to_be_bytes().as_slice());
        assert_eq!(&payload[24..32], 500000u64.to_be_bytes().as_slice());
    }
}
