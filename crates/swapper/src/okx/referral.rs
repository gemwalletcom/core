use crate::{fee_token::FeeTokenPriority, fees::default_referral_address};
use primitives::{Chain, swap::QuoteAsset};

pub(super) struct ReferrerWalletAddresses {
    pub(super) from_token: Option<String>,
    pub(super) to_token: Option<String>,
}

pub(super) fn referrer_wallet_addresses(from_asset: &QuoteAsset, to_asset: &QuoteAsset, chain: Chain) -> ReferrerWalletAddresses {
    let referrer = default_referral_address(chain);
    if prefer_input_as_fee_token(from_asset, to_asset) {
        ReferrerWalletAddresses {
            from_token: Some(referrer),
            to_token: None,
        }
    } else {
        ReferrerWalletAddresses {
            from_token: None,
            to_token: Some(referrer),
        }
    }
}

fn fee_token_priority(asset: &QuoteAsset) -> FeeTokenPriority {
    let asset_id = asset.asset_id();
    FeeTokenPriority::from_asset(&asset_id, &asset.symbol)
}

fn prefer_input_as_fee_token(from_asset: &QuoteAsset, to_asset: &QuoteAsset) -> bool {
    fee_token_priority(from_asset).rank() > fee_token_priority(to_asset).rank()
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{
        AssetId, EVMChain,
        asset_constants::{SMARTCHAIN_CAKE_TOKEN_ID, SMARTCHAIN_USDC_TOKEN_ID, SOLANA_USDC_ASSET_ID},
        contract_constants::SOLANA_WRAPPED_SOL_TOKEN_ADDRESS,
    };

    const SOLANA_BONK_TOKEN_ID: &str = "DezXAZ8z7PnrnRJjz3RhxPntipBqJJfMaB3Uy7VxwkL";

    #[test]
    fn test_prefer_input_as_fee_token() {
        let bnb = QuoteAsset::mock_with_asset_id(AssetId::from_chain(Chain::SmartChain), "BNB", 18);
        let wbnb = QuoteAsset::mock_with_asset_id(AssetId::from_token(Chain::SmartChain, EVMChain::SmartChain.weth_contract().unwrap()), "WBNB", 18);
        let usdc = QuoteAsset::mock_with_asset_id(AssetId::from_token(Chain::SmartChain, SMARTCHAIN_USDC_TOKEN_ID), "USDC", 6);
        let cake = QuoteAsset::mock_with_asset_id(AssetId::from_token(Chain::SmartChain, SMARTCHAIN_CAKE_TOKEN_ID), "CAKE", 18);
        let wsol = QuoteAsset::mock_with_asset_id(AssetId::from_token(Chain::Solana, SOLANA_WRAPPED_SOL_TOKEN_ADDRESS), "SOL", 9);
        let bonk = QuoteAsset::mock_with_asset_id(AssetId::from_token(Chain::Solana, SOLANA_BONK_TOKEN_ID), "BONK", 5);

        assert!(prefer_input_as_fee_token(&bnb, &cake));
        assert!(!prefer_input_as_fee_token(&cake, &bnb));
        assert!(prefer_input_as_fee_token(&wbnb, &cake));
        assert!(!prefer_input_as_fee_token(&cake, &wbnb));
        assert!(prefer_input_as_fee_token(&usdc, &cake));
        assert!(!prefer_input_as_fee_token(&cake, &usdc));
        assert!(prefer_input_as_fee_token(&wbnb, &usdc));
        assert!(!prefer_input_as_fee_token(&usdc, &wbnb));
        assert!(!prefer_input_as_fee_token(&cake, &cake));
        assert!(prefer_input_as_fee_token(&wsol, &bonk));
        assert!(!prefer_input_as_fee_token(&bonk, &wsol));
    }

    #[test]
    fn test_referrer_wallet_addresses() {
        let bnb = QuoteAsset::mock_with_asset_id(AssetId::from_chain(Chain::SmartChain), "BNB", 18);
        let cake = QuoteAsset::mock_with_asset_id(AssetId::from_token(Chain::SmartChain, SMARTCHAIN_CAKE_TOKEN_ID), "CAKE", 18);
        let evm_referrer = default_referral_address(Chain::SmartChain);

        let input_referrer = referrer_wallet_addresses(&bnb, &cake, Chain::SmartChain);
        assert_eq!(input_referrer.from_token.as_deref(), Some(evm_referrer.as_str()));
        assert_eq!(input_referrer.to_token, None);

        let output_referrer = referrer_wallet_addresses(&cake, &bnb, Chain::SmartChain);
        assert_eq!(output_referrer.from_token, None);
        assert_eq!(output_referrer.to_token.as_deref(), Some(evm_referrer.as_str()));

        let sol = QuoteAsset::mock_with_asset_id(AssetId::from_chain(Chain::Solana), "SOL", 9);
        let usdc = QuoteAsset::mock_with_asset_id(SOLANA_USDC_ASSET_ID.clone(), "USDC", 6);
        let solana_referrer = default_referral_address(Chain::Solana);
        let solana_input_referrer = referrer_wallet_addresses(&sol, &usdc, Chain::Solana);
        assert_eq!(solana_input_referrer.from_token.as_deref(), Some(solana_referrer.as_str()));
        assert_eq!(solana_input_referrer.to_token, None);
    }
}
