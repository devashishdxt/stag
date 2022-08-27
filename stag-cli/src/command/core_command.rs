use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Context, Result};
use clap::{Subcommand, ValueEnum};
use stag_api::types::{
    chain_state::{ChainConfig, Fee},
    ics::core::ics24_host::identifier::{ChainId, PortId},
    public_key::{PublicKey, PublicKeyAlgo},
};

use crate::util::stag;

#[derive(Debug, Subcommand)]
pub enum CoreCommand {
    /// Prints sample chain configuration
    SampleChainConfig,
    /// Adds an IBC enabled chain to the solo machine
    AddChain { config: PathBuf },
    /// Establishes an IBC connection with given chain
    Connect {
        /// Chain ID
        chain_id: ChainId,
        /// An optional request ID for tracking purposes
        #[clap(long)]
        request_id: Option<String>,
        /// Memo value to be used in cosmos sdk transaction
        #[clap(long)]
        memo: Option<String>,
        /// Force create a new connection even if one already exists
        #[clap(short, long)]
        force: bool,
    },
    /// Channel commands
    Channel {
        #[clap(subcommand)]
        subcommand: ChannelCommand,
    },
    /// Updates signer for future IBC transactions
    UpdateSigner {
        /// Chain ID
        chain_id: ChainId,
        /// An optional request ID for tracking purposes
        #[clap(long)]
        request_id: Option<String>,
        /// Hex encoded SEC1 bytes of public key
        #[clap(short, long)]
        new_public_key: String,
        /// Algorithm used for address generation
        #[clap(short, long, default_value = "secp256k1")]
        algo: PublicKeyAlgo,
        /// Memo value to be used in cosmos sdk transaction
        #[clap(long)]
        memo: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum ChannelCommand {
    /// Create a new IBC channel
    Create {
        /// Type of channel to create
        #[clap(value_enum)]
        channel_type: ChannelType,
        /// Chain ID
        chain_id: ChainId,
        /// An optional request ID for tracking purposes
        #[clap(long)]
        request_id: Option<String>,
        /// Memo value to be used in cosmos sdk transaction
        #[clap(long)]
        memo: Option<String>,
    },
    /// Close an existing IBC channel
    Close {
        /// Type of channel to close
        #[clap(value_enum)]
        channel_type: ChannelType,
        /// Chain ID
        chain_id: ChainId,
        /// An optional request ID for tracking purposes
        #[clap(long)]
        request_id: Option<String>,
        /// Memo value to be used in cosmos sdk transaction
        #[clap(long)]
        memo: Option<String>,
    },
}

/// Channel type
#[derive(Debug, ValueEnum, Clone, Copy)]
pub enum ChannelType {
    /// Transfer channel
    Transfer,
    /// ICA channel
    Ica,
}

impl CoreCommand {
    pub async fn run(self, signer: impl AsRef<Path>, db_uri: &str) -> Result<()> {
        match self {
            Self::SampleChainConfig => {
                let chain_config = get_sample_config();
                let output = serde_yaml::to_string(&chain_config)
                    .context("unable to serialize sample config")?;

                println!("{}", output);
                Ok(())
            }
            Self::AddChain { config } => {
                let chain_config_bytes = tokio::fs::read(&config)
                    .await
                    .context(format!("failed to read {}", config.display()))?;

                let chain_config: ChainConfig = serde_yaml::from_slice(&chain_config_bytes)
                    .context(format!("failed to deserialize {}", config.display()))?;

                let chain_id = stag(signer, db_uri)
                    .await?
                    .add_chain(&chain_config)
                    .await
                    .context(format!("failed to add chain from {}", config.display()))?;

                println!("successfully added chain with id: {}", chain_id);

                Ok(())
            }
            Self::Connect {
                chain_id,
                request_id,
                memo,
                force,
            } => {
                stag(signer, db_uri)
                    .await?
                    .connect(
                        chain_id.clone(),
                        request_id,
                        memo.unwrap_or_default(),
                        force,
                    )
                    .await?;

                println!("successfully established IBC connection with {}", chain_id);

                Ok(())
            }
            Self::Channel { subcommand } => subcommand.run(signer, db_uri).await,
            Self::UpdateSigner {
                chain_id,
                request_id,
                new_public_key,
                algo,
                memo,
            } => {
                let new_public_key = PublicKey::new(new_public_key, algo)?;

                stag(signer, db_uri)
                    .await?
                    .update_signer(
                        chain_id.clone(),
                        request_id,
                        new_public_key,
                        memo.unwrap_or_default(),
                    )
                    .await?;

                println!("successfully updated signer for {}", chain_id);

                Ok(())
            }
        }
    }
}

impl ChannelCommand {
    pub async fn run(self, signer: impl AsRef<Path>, db_uri: &str) -> Result<()> {
        match self {
            Self::Create {
                channel_type,
                chain_id,
                request_id,
                memo,
            } => match channel_type {
                ChannelType::Transfer => {
                    stag(signer, db_uri)
                        .await?
                        .create_transfer_channel(
                            chain_id.clone(),
                            request_id,
                            memo.unwrap_or_default(),
                        )
                        .await?;

                    println!("successfully created transfer channel on {}", chain_id);

                    Ok(())
                }
                ChannelType::Ica => {
                    stag(signer, db_uri)
                        .await?
                        .create_ica_channel(chain_id.clone(), request_id, memo.unwrap_or_default())
                        .await?;

                    println!("successfully created ICA channel on {}", chain_id);

                    Ok(())
                }
            },
            Self::Close {
                channel_type,
                chain_id,
                request_id,
                memo,
            } => {
                let port_id = match channel_type {
                    ChannelType::Transfer => PortId::transfer(),
                    ChannelType::Ica => PortId::ica_controller(),
                };

                stag(signer, db_uri)
                    .await?
                    .close_channel(
                        chain_id.clone(),
                        &port_id,
                        request_id,
                        memo.unwrap_or_default(),
                    )
                    .await?;

                println!(
                    "successfully close channel with port {} on {}",
                    port_id, chain_id
                );

                Ok(())
            }
        }
    }
}

fn get_sample_config() -> ChainConfig {
    ChainConfig {
        grpc_addr: "http://0.0.0.0:9090".parse().unwrap(),
        rpc_addr: "http://0.0.0.0:26657".parse().unwrap(),
        fee: Fee {
            amount: "1000".parse().unwrap(),
            denom: "stake".parse().unwrap(),
            gas_limit: 300000,
        },
        trust_level: "1/3".parse().unwrap(),
        trusting_period: Duration::from_secs(336 * 60 * 60),
        max_clock_drift: Duration::from_secs(3),
        rpc_timeout: Duration::from_secs(60),
        diversifier: "stag".to_string(),
        trusted_height: 1,
        trusted_hash: [0; 32],
        packet_timeout_height_offset: 20,
    }
}
