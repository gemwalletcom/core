use clap::Args;
use primitives::Chain;
use settings_chain::ChainProviders;
use std::error::Error;

#[derive(Args)]
pub struct BalanceCommand {
    chain: Chain,
    address: String,
}

impl BalanceCommand {
    pub async fn run(&self, providers: &ChainProviders) -> Result<(), Box<dyn Error + Send + Sync>> {
        let chain = self.chain;
        let address = &self.address;

        match providers.get_balance_coin(chain, address.to_string()).await {
            Ok(balance) => println!("{}: {}", balance.asset_id, balance.balance.available),
            Err(e) => eprintln!("Coin balance error: {}", e),
        }

        match providers.get_balance_assets(chain, address.to_string()).await {
            Ok(balances) => {
                for balance in balances {
                    println!("{}: {}", balance.asset_id, balance.balance.available);
                }
            }
            Err(e) => eprintln!("Assets balance error: {}", e),
        }

        match providers.get_balance_staking(chain, address.to_string()).await {
            Ok(Some(balance)) => println!("{} (staked): {}", balance.asset_id, balance.balance.staked),
            Ok(None) => {}
            Err(e) => eprintln!("Staking balance error: {}", e),
        }

        Ok(())
    }
}
