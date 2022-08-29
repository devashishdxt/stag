pub mod core_command;
pub mod ica_command;
pub mod query_command;
pub mod signer_command;
pub mod transfer_command;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

use self::{
    core_command::CoreCommand, ica_command::IcaCommand, query_command::QueryCommand,
    signer_command::SignerCommand, transfer_command::TransferCommand,
};

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Command {
    /// Path to signer.yaml file for configuration of mnemonic-signer
    #[clap(
        short,
        long,
        default_value = "signer.yaml",
        env = "SOLO_SIGNER",
        global = true
    )]
    signer: PathBuf,
    /// Database connection string
    #[clap(short, long, env = "SOLO_DB_URI", global = true)]
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
    /// Transfer channel commands
    Transfer {
        #[clap(subcommand)]
        subcommand: TransferCommand,
    },
    /// ICA channel commands
    Ica {
        #[clap(subcommand)]
        subcommand: IcaCommand,
    },
    /// Query on-chain data
    Query {
        #[clap(subcommand)]
        subcommand: QueryCommand,
    },
}

impl SubCommand {
    pub async fn run(self, signer: PathBuf, db_uri: String) -> Result<()> {
        match self {
            Self::Signer { subcommand } => subcommand.run(signer).await,
            Self::Core { subcommand } => subcommand.run(signer, &db_uri).await,
            Self::Transfer { subcommand } => subcommand.run(signer, &db_uri).await,
            Self::Ica { subcommand } => subcommand.run(signer, &db_uri).await,
            Self::Query { subcommand } => subcommand.run(signer, &db_uri).await,
        }
    }
}
