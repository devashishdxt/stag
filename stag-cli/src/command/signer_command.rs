use std::path::Path;

use anyhow::{Context, Result};
use clap::Subcommand;

use crate::signer_config::{ChainSignerConfig, SignerConfig};

#[derive(Debug, Subcommand)]
pub enum SignerCommand {
    /// Prints sample signer configuration
    SampleConfig,
    /// Validates signer configuration
    ValidateConfig,
}

impl SignerCommand {
    /// Runs signer command
    pub async fn run(self, path: impl AsRef<Path>) -> Result<()> {
        match self {
            Self::SampleConfig => {
                let signer_config = get_sample_config();
                let output = serde_yaml::to_string(&signer_config)
                    .context("unable to serialize sample config")?;

                println!("{}", output);
                Ok(())
            }
            Self::ValidateConfig => {
                SignerConfig::create_mnemonic_signer(&path).await?;
                println!("{} is valid", path.as_ref().display());
                Ok(())
            }
        }
    }
}

fn get_sample_config() -> SignerConfig {
    SignerConfig {
        chains: vec![ChainSignerConfig {
            chain_id: "mars-1".parse().unwrap(),
            mnemonic: "practice empty client sauce pistol work ticket casual romance appear army fault palace coyote fox super salute slim catch kite wrist three hedgehog sign".to_string(),
            hd_path: None,
            account_prefix: None,
            algo: None,
        }, ChainSignerConfig {
            chain_id: "mars-2".parse().unwrap(),
            mnemonic: "setup chicken slogan define emerge original sugar bitter suggest bicycle increase eager rather end predict relief moment burden lonely ginger umbrella secret toy trash".to_string(),
            hd_path: None,
            account_prefix: None,
            algo: None,
        }],
    }
}
