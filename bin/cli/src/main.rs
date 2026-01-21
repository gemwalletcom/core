mod commands;

use clap::{Parser, Subcommand};
use commands::{asset::AssetCommand, balance::BalanceCommand};
use settings::Settings;
use settings_chain::ChainProviders;
use std::error::Error;

#[derive(Parser)]
#[command(name = "cli")]
#[command(about = "Gem Wallet CLI tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Asset(AssetCommand),
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
