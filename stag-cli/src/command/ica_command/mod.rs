pub mod bank_command;
pub mod staking_command;

use std::path::Path;

use anyhow::Result;
use clap::Subcommand;

use self::{bank_command::BankCommand, staking_command::StakingCommand};

#[derive(Debug, Subcommand)]
pub enum IcaCommand {
    /// Bank module transactions
    Bank {
        #[clap(subcommand)]
        subcommand: BankCommand,
    },
    /// Staking module transactions
    Staking {
        #[clap(subcommand)]
        subcommand: StakingCommand,
    },
}

impl IcaCommand {
    pub async fn run(self, signer: impl AsRef<Path>, db_uri: &str) -> Result<()> {
        match self {
            Self::Bank { subcommand } => subcommand.run(signer, db_uri).await,
            Self::Staking { subcommand } => subcommand.run(signer, db_uri).await,
        }
    }
}
