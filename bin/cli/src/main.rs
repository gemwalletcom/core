mod commands;

use clap::{Parser, Subcommand};
use commands::{asset::AssetCommand, balance::BalanceCommand};
use settings::Settings;
use settings_chain::ChainProviders;
use std::error::Error;

#[derive(Parser)]
#[command(name = "cli")]
#[command(about = "Gem Wallet CLI tool")]
#[command(after_help = "Examples:
  cli asset info solana Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB
  cli balance solana 7v91N7iZ9mNicL8WfG6cgSCKyRXydQjLh6UYBWwm6y1Q
")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get token info
    Asset(AssetCommand),
    /// Get address balances
    Balance(BalanceCommand),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let cli = Cli::parse();
    let settings = Settings::new()?;
    let providers = ChainProviders::from_settings(&settings, "cli");

    match cli.command {
        Commands::Asset(cmd) => cmd.run(&providers).await,
        Commands::Balance(cmd) => cmd.run(&providers).await,
    }
}
