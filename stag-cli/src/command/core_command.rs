use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Context, Result};
use clap::Subcommand;
use stag_api::types::chain_state::{ChainConfig, Fee};

use crate::util::stag;

#[derive(Debug, Subcommand)]
pub enum CoreCommand {
    /// Prints sample chain configuration
    SampleChainConfig,
    /// Adds an IBC enabled chain to the solo machine
    AddChain { config: PathBuf },
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
