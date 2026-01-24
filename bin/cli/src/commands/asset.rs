use clap::{Args, Subcommand};
use primitives::Chain;
use settings_chain::ChainProviders;
use std::error::Error;

#[derive(Args)]
pub struct AssetCommand {
    #[command(subcommand)]
    command: AssetSubcommand,
}

#[derive(Subcommand)]
enum AssetSubcommand {
    /// Get token info
    Info {
        /// Chain name (e.g., solana, ethereum)
        chain: Chain,
        /// Token ID / contract address
        token_id: String,
    },
}

impl AssetCommand {
    pub async fn run(&self, providers: &ChainProviders) -> Result<(), Box<dyn Error + Send + Sync>> {
        match &self.command {
            AssetSubcommand::Info { chain, token_id } => self.info(providers, *chain, token_id).await,
        }
    }

    async fn info(&self, providers: &ChainProviders, chain: Chain, token_id: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let asset = providers.get_token_data(chain, token_id.to_string()).await?;

        println!("Asset ID: {}", asset.id);
        println!("Name: {}", asset.name);
        println!("Symbol: {}", asset.symbol);
        println!("Decimals: {}", asset.decimals);
        println!("Type: {:?}", asset.asset_type);

        Ok(())
    }
}
