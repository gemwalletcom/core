use std::collections::{HashMap, HashSet};
use std::error::Error;

use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::STANDARD};
use chain_traits::ChainSimulation;
use gem_client::Client;
use num_bigint::BigInt;
use primitives::{AssetId, BalanceDiff, BalanceDiffMap, Chain, SimulationInput, SimulationResult};
use solana_primitives::VersionedTransaction;

use crate::models::token::TokenBalance;
use crate::rpc::client::SolanaClient;

#[async_trait]
impl<C: Client + Clone> ChainSimulation for SolanaClient<C> {
    async fn simulate_transaction(&self, input: SimulationInput) -> Result<SimulationResult, Box<dyn Error + Send + Sync>> {
        let tx_bytes = STANDARD.decode(&input.encoded_transaction).map_err(|e| format!("invalid base64: {e}"))?;

        let transaction = VersionedTransaction::deserialize_with_version(&tx_bytes).map_err(|e| format!("parse transaction: {e}"))?;

        let account_keys: Vec<String> = transaction.account_keys().iter().map(|k| k.to_string()).collect();

        let sim_result = self.simulate_transaction(&input.encoded_transaction).await?;

        if let Some(err) = &sim_result.err {
            return Err(err.to_string().into());
        }

        let balance_changes = map_balance_changes(
            &account_keys,
            &sim_result.pre_balances,
            &sim_result.post_balances,
            &sim_result.pre_token_balances.unwrap_or_default(),
            &sim_result.post_token_balances.unwrap_or_default(),
        );

        Ok(SimulationResult {
            success: true,
            error: None,
            logs: sim_result.logs.unwrap_or_default(),
            units_consumed: sim_result.units_consumed,
            balance_changes,
        })
    }
}

fn map_balance_changes(
    account_keys: &[String],
    pre_balances: &[u64],
    post_balances: &[u64],
    pre_token_balances: &[TokenBalance],
    post_token_balances: &[TokenBalance],
) -> BalanceDiffMap {
    let mut result: BalanceDiffMap = HashMap::new();

    for (i, address) in account_keys.iter().enumerate() {
        let pre = pre_balances.get(i).copied().unwrap_or(0);
        let post = post_balances.get(i).copied().unwrap_or(0);
        if pre != post {
            result.entry(address.clone()).or_default().push(BalanceDiff {
                asset_id: AssetId::from_chain(Chain::Solana),
                from_value: Some(BigInt::from(pre)),
                to_value: Some(BigInt::from(post)),
                diff: BigInt::from(post as i128 - pre as i128),
            });
        }
    }

    let owners: HashSet<&str> = pre_token_balances.iter().chain(post_token_balances.iter()).map(|tb| tb.owner.as_str()).collect();

    for owner in owners {
        let pre_by_mint: HashMap<&str, &TokenBalance> = pre_token_balances.iter().filter(|tb| tb.owner == owner).map(|tb| (tb.mint.as_str(), tb)).collect();

        let post_by_mint: HashMap<&str, &TokenBalance> = post_token_balances.iter().filter(|tb| tb.owner == owner).map(|tb| (tb.mint.as_str(), tb)).collect();

        let all_mints: HashSet<&str> = pre_by_mint.keys().chain(post_by_mint.keys()).copied().collect();

        for mint in all_mints {
            let pre_amount = pre_by_mint.get(mint).map(|tb| BigInt::from(tb.get_amount())).unwrap_or_default();
            let post_amount = post_by_mint.get(mint).map(|tb| BigInt::from(tb.get_amount())).unwrap_or_default();
            let diff = &post_amount - &pre_amount;

            if diff != BigInt::from(0) {
                result.entry(owner.to_string()).or_default().push(BalanceDiff {
                    asset_id: AssetId::from_token(Chain::Solana, mint),
                    from_value: Some(pre_amount),
                    to_value: Some(post_amount),
                    diff,
                });
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigUint;

    use crate::models::token::TokenAmount;

    fn make_token_balance(account_index: i64, mint: &str, owner: &str, amount: u64) -> TokenBalance {
        TokenBalance::new(account_index, mint.to_string(), owner.to_string(), TokenAmount { amount: BigUint::from(amount) })
    }

    #[test]
    fn test_native_sol_balance_change() {
        let keys = vec!["addr1".to_string()];
        let pre = vec![1_000_000_000u64];
        let post = vec![900_000_000u64];

        let changes = map_balance_changes(&keys, &pre, &post, &[], &[]);

        let diffs = changes.get("addr1").unwrap();
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].asset_id, AssetId::from_chain(Chain::Solana));
        assert_eq!(diffs[0].from_value, Some(BigInt::from(1_000_000_000)));
        assert_eq!(diffs[0].to_value, Some(BigInt::from(900_000_000)));
        assert_eq!(diffs[0].diff, BigInt::from(-100_000_000));
    }

    #[test]
    fn test_no_change() {
        let keys = vec!["addr1".to_string()];
        let pre = vec![1_000_000_000u64];
        let post = vec![1_000_000_000u64];

        let changes = map_balance_changes(&keys, &pre, &post, &[], &[]);
        assert!(changes.is_empty());
    }

    #[test]
    fn test_spl_token_balance_change() {
        let mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
        let owner = "wallet_owner";
        let keys = vec!["token_account_addr".to_string()];
        let pre_balances = vec![2_039_280u64];
        let post_balances = vec![2_039_280u64];
        let pre_tokens = vec![make_token_balance(0, mint, owner, 1_000_000)];
        let post_tokens = vec![make_token_balance(0, mint, owner, 500_000)];

        let changes = map_balance_changes(&keys, &pre_balances, &post_balances, &pre_tokens, &post_tokens);

        let diffs = changes.get(owner).unwrap();
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].asset_id, AssetId::from_token(Chain::Solana, mint));
        assert_eq!(diffs[0].diff, BigInt::from(-500_000));
    }

    #[test]
    fn test_new_token_account_created() {
        let mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
        let owner = "wallet_owner";
        let keys = vec!["new_token_addr".to_string()];
        let pre_balances = vec![0u64];
        let post_balances = vec![2_039_280u64];
        let pre_tokens: Vec<TokenBalance> = vec![];
        let post_tokens = vec![make_token_balance(0, mint, owner, 1_000_000)];

        let changes = map_balance_changes(&keys, &pre_balances, &post_balances, &pre_tokens, &post_tokens);

        let sol_diffs = changes.get("new_token_addr").unwrap();
        assert_eq!(sol_diffs.len(), 1);
        assert_eq!(sol_diffs[0].asset_id, AssetId::from_chain(Chain::Solana));

        let token_diffs = changes.get(owner).unwrap();
        assert_eq!(token_diffs.len(), 1);
        assert_eq!(token_diffs[0].asset_id, AssetId::from_token(Chain::Solana, mint));
        assert_eq!(token_diffs[0].from_value, Some(BigInt::from(0)));
        assert_eq!(token_diffs[0].to_value, Some(BigInt::from(1_000_000)));
        assert_eq!(token_diffs[0].diff, BigInt::from(1_000_000));
    }

    #[test]
    fn test_multiple_accounts_mixed() {
        let mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
        let keys = vec!["sol_addr".to_string(), "token_addr".to_string()];
        let pre_balances = vec![1_000_000_000u64, 2_039_280];
        let post_balances = vec![900_000_000u64, 2_039_280];
        let pre_tokens = vec![make_token_balance(1, mint, "wallet_owner", 1_000_000)];
        let post_tokens = vec![make_token_balance(1, mint, "wallet_owner", 2_000_000)];

        let changes = map_balance_changes(&keys, &pre_balances, &post_balances, &pre_tokens, &post_tokens);

        let sol_diffs = changes.get("sol_addr").unwrap();
        assert_eq!(sol_diffs.len(), 1);
        assert_eq!(sol_diffs[0].asset_id, AssetId::from_chain(Chain::Solana));
        assert_eq!(sol_diffs[0].diff, BigInt::from(-100_000_000));

        let token_diffs = changes.get("wallet_owner").unwrap();
        assert_eq!(token_diffs.len(), 1);
        assert_eq!(token_diffs[0].asset_id, AssetId::from_token(Chain::Solana, mint));
        assert_eq!(token_diffs[0].diff, BigInt::from(1_000_000));
    }
}
