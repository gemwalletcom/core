use super::{
    asset::asset_id_for_token,
    model::{MayanClientStatus, MayanTransactionResult},
    wormhole_chain,
};
use crate::{SwapResult, SwapperProvider};
use primitives::TransactionSwapMetadata;

pub fn map_swap_result(result: &MayanTransactionResult) -> SwapResult {
    let status = result.client_status.swap_status();

    let from_chain = result.from_token_chain.parse::<u16>().ok().and_then(wormhole_chain::chain_from_id);
    let to_chain = result.to_token_chain.parse::<u16>().ok().and_then(wormhole_chain::chain_from_id);

    let metadata = if result.client_status != MayanClientStatus::InProgress {
        from_chain.zip(to_chain).and_then(|(from_chain, to_chain)| {
            Some(TransactionSwapMetadata {
                from_asset: asset_id_for_token(from_chain, &result.from_token_address)?,
                from_value: result.from_amount64.clone()?,
                to_asset: asset_id_for_token(to_chain, &result.to_token_address)?,
                to_value: result.to_amount64.clone()?,
                provider: Some(SwapperProvider::Mayan.as_ref().to_string()),
            })
        })
    } else {
        None
    };

    SwapResult { status, metadata }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{AssetId, Chain, asset_constants::POLYGON_USDT_ASSET_ID, swap::SwapStatus};

    fn result(json: &str) -> MayanTransactionResult {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn test_map_swap_result() {
        let missing_to_amount64 = map_swap_result(&result(include_str!("test/eth_to_sui_swift.json")));
        assert_eq!(missing_to_amount64.status, SwapStatus::Completed);
        assert!(missing_to_amount64.metadata.is_none());

        assert_eq!(
            map_swap_result(&result(include_str!("test/pol_to_bnb_swift.json"))),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(TransactionSwapMetadata {
                    from_asset: AssetId::from_chain(Chain::Polygon),
                    from_value: "21782666".to_string(),
                    to_asset: AssetId::from_chain(Chain::SmartChain),
                    to_value: "33060513057817862".to_string(),
                    provider: Some("mayan".to_string()),
                }),
            }
        );
        assert_eq!(
            map_swap_result(&result(include_str!("test/usdt_to_owb_swift.json"))),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(TransactionSwapMetadata {
                    from_asset: POLYGON_USDT_ASSET_ID.clone(),
                    from_value: "35245466".to_string(),
                    to_asset: AssetId::from_token(Chain::Base, "0xEF5997c2cf2f6c138196f8A6203afc335206b3c1"),
                    to_value: "398724622644505839482".to_string(),
                    provider: Some("mayan".to_string()),
                }),
            }
        );
        assert_eq!(
            map_swap_result(&result(include_str!("test/btcbr_to_radr_swift.json"))),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(TransactionSwapMetadata {
                    from_asset: AssetId::from_token(Chain::SmartChain, "0x0cF8e180350253271f4b917CcFb0aCCc4862F262"),
                    from_value: "10686571736749000000".to_string(),
                    to_asset: AssetId::from_token(Chain::Solana, "CzFvsLdUazabdiu9TYXujj4EY495fG7VgJJ3vQs6bonk"),
                    to_value: "278080608518046".to_string(),
                    provider: Some("mayan".to_string()),
                }),
            }
        );

        let pending = map_swap_result(&result(include_str!("test/mctp_pending.json")));
        assert_eq!(pending.status, SwapStatus::Pending);
        assert!(pending.metadata.is_none());

        let refunded = map_swap_result(&result(include_str!("test/swift_refunded.json")));
        assert_eq!(refunded.status, SwapStatus::Failed);
        assert!(refunded.metadata.is_none());
    }
}
