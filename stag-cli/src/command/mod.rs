pub mod core_command;
pub mod signer_command;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

use self::{core_command::CoreCommand, signer_command::SignerCommand};

#[derive(Debug, Parser)]
pub struct Command {
    /// Path to signer.yaml file for configuration of mnemonic-signer
    #[clap(short, long, default_value = "signer.yaml", global = true)]
    signer: PathBuf,
    /// Database connection string
    #[clap(short, long, global = true)]
    #[cfg_attr(feature = "sqlite-storage", clap(default_value = "sqlite://stag.db"))]
    #[cfg_attr(
        feature = "postgres-storage",
        clap(default_value = "postgresql://postgres:postgres@localhost:5432/stag")
    )]
    db_uri: String,
    #[clap(subcommand)]
    subcommand: SubCommand,
}

impl Command {
    pub async fn run(self) -> Result<()> {
        self.subcommand.run(self.signer, self.db_uri).await
    }
}

#[derive(Debug, Subcommand)]
pub enum SubCommand {
    /// Signer commands
    Signer {
        #[clap(subcommand)]
        subcommand: SignerCommand,
    },
    /// Core commands
    Core {
        #[clap(subcommand)]
        subcommand: CoreCommand,
    },
}

impl SubCommand {
    pub async fn run(self, signer: PathBuf, db_uri: String) -> Result<()> {
        match self {
            Self::Signer { subcommand } => subcommand.run(signer).await,
            Self::Core { subcommand } => subcommand.run(signer, &db_uri).await,
        }
    }
}
