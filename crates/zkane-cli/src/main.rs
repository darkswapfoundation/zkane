//! # ZKane CLI
//!
//! The main entry point for the ZKane privacy pool CLI.

use anyhow::Result;
use clap::Parser;
use deezel_common::traits::DeezelProvider;
use deezel_common::System;
use deezel_sys::SystemDeezel;
use std::sync::Arc;
use zkane_common::ZKaneConfig;
use zkane_core::PrivacyPool;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(flatten)]
    pub deezel_args: deezel_common::commands::Args,

    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Parser)]
pub enum Commands {
    /// Deposit funds into the privacy pool
    Deposit,
    /// Withdraw funds from the privacy pool
    Withdraw,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = Args::parse();

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(&args.deezel_args.log_level))
        .init();

    let deezel = SystemDeezel::new(&args.deezel_args).await?;
    let config = ZKaneConfig::new(
        zkane_common::SerializableAlkaneId { block: 0, tx: 0 }, // Placeholder
        1000000,
        20,
        vec![],
    );
    let _zkane_pool = PrivacyPool::new(config, Arc::new(deezel.provider().clone_box()));

    match args.command {
        Commands::Deposit => {
            println!("Depositing funds...");
        }
        Commands::Withdraw => {
            println!("Withdrawing funds...");
        }
    }

    Ok(())
}