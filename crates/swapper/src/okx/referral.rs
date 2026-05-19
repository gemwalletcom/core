use crate::{fee_token::FeeTokenPriority, fees::default_referral_fees};
use primitives::{Chain, swap::QuoteAsset};

pub(super) struct ReferrerWalletAddresses {
    pub(super) from_token: Option<String>,
    pub(super) to_token: Option<String>,
}

pub(super) fn referrer_wallet_addresses(from_asset: &QuoteAsset, to_asset: &QuoteAsset, referral_bps: u32, chain: Chain) -> ReferrerWalletAddresses {
    if referral_bps == 0 {
        return ReferrerWalletAddresses { from_token: None, to_token: None };
    }
    let referrer = referrer_wallet(chain);
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

fn referrer_wallet(chain: Chain) -> String {
    let fees = default_referral_fees();
    match chain {
        Chain::Solana => fees.solana.address,
        _ => fees.evm.address,
    }
}

fn fee_token_priority(asset: &QuoteAsset) -> FeeTokenPriority {
    let asset_id = asset.asset_id();
    FeeTokenPriority::from_asset(&asset_id, &asset.symbol)
}

fn prefer_input_as_fee_token(from_asset: &QuoteAsset, to_asset: &QuoteAsset) -> bool {
    fee_token_priority(from_asset).rank() >= fee_token_priority(to_asset).rank()
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{
        AssetId, EVMChain,
        asset_constants::{SMARTCHAIN_USDC_TOKEN_ID, SOLANA_USDC_ASSET_ID},
        contract_constants::SOLANA_WRAPPED_SOL_TOKEN_ADDRESS,
    };

    const SMARTCHAIN_FAKE_BTC_TOKEN_ID: &str = "0x4770Ab6fCed223124b8616e08003D37fF13F8888";

    fn asset(id: AssetId, symbol: &str) -> QuoteAsset {
        QuoteAsset {
            id: id.to_string(),
            symbol: symbol.to_string(),
            decimals: 18,
        }
    }

    #[test]
    fn test_prefer_input_as_fee_token() {
        let bnb = asset(AssetId::from_chain(Chain::SmartChain), "BNB");
        let wbnb = asset(AssetId::from_token(Chain::SmartChain, EVMChain::SmartChain.weth_contract().unwrap()), "WBNB");
        let usdc = asset(AssetId::from_token(Chain::SmartChain, SMARTCHAIN_USDC_TOKEN_ID), "USDC");
        let fake_btc = asset(AssetId::from_token(Chain::SmartChain, SMARTCHAIN_FAKE_BTC_TOKEN_ID), "BTC");
        let wsol = asset(AssetId::from_token(Chain::Solana, SOLANA_WRAPPED_SOL_TOKEN_ADDRESS), "SOL");
        let sol_fake = asset(AssetId::from_token(Chain::Solana, "Fake111111111111111111111111111111111111111"), "FAKE");

        assert!(prefer_input_as_fee_token(&bnb, &fake_btc));
        assert!(!prefer_input_as_fee_token(&fake_btc, &bnb));
        assert!(prefer_input_as_fee_token(&wbnb, &fake_btc));
        assert!(!prefer_input_as_fee_token(&fake_btc, &wbnb));
        assert!(prefer_input_as_fee_token(&usdc, &fake_btc));
        assert!(!prefer_input_as_fee_token(&fake_btc, &usdc));
        assert!(prefer_input_as_fee_token(&wbnb, &usdc));
        assert!(!prefer_input_as_fee_token(&usdc, &wbnb));
        assert!(prefer_input_as_fee_token(&fake_btc, &fake_btc));
        assert!(prefer_input_as_fee_token(&wsol, &sol_fake));
        assert!(!prefer_input_as_fee_token(&sol_fake, &wsol));
    }

    #[test]
    fn test_referrer_wallet_addresses() {
        let bnb = asset(AssetId::from_chain(Chain::SmartChain), "BNB");
        let fake_btc = asset(AssetId::from_token(Chain::SmartChain, SMARTCHAIN_FAKE_BTC_TOKEN_ID), "BTC");
        let evm_referrer = default_referral_fees().evm.address;

        let input_referrer = referrer_wallet_addresses(&bnb, &fake_btc, 70, Chain::SmartChain);
        assert_eq!(input_referrer.from_token.as_deref(), Some(evm_referrer.as_str()));
        assert_eq!(input_referrer.to_token, None);

        let output_referrer = referrer_wallet_addresses(&fake_btc, &bnb, 70, Chain::SmartChain);
        assert_eq!(output_referrer.from_token, None);
        assert_eq!(output_referrer.to_token.as_deref(), Some(evm_referrer.as_str()));

        let no_fee_referrer = referrer_wallet_addresses(&bnb, &fake_btc, 0, Chain::SmartChain);
        assert_eq!(no_fee_referrer.from_token, None);
        assert_eq!(no_fee_referrer.to_token, None);

        let sol = asset(AssetId::from_chain(Chain::Solana), "SOL");
        let usdc = asset(SOLANA_USDC_ASSET_ID.clone(), "USDC");
        let solana_referrer = default_referral_fees().solana.address;
        let solana_input_referrer = referrer_wallet_addresses(&sol, &usdc, 50, Chain::Solana);
        assert_eq!(solana_input_referrer.from_token.as_deref(), Some(solana_referrer.as_str()));
        assert_eq!(solana_input_referrer.to_token, None);
    }
}
