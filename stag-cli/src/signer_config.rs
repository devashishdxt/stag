use std::{path::Path, str::FromStr};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use stag_api::{signer::MnemonicSigner, types::ics::core::ics24_host::identifier::ChainId};

#[derive(Debug, Serialize, Deserialize)]
pub struct SignerConfig {
    /// Signer config for all chains
    pub chains: Vec<ChainSignerConfig>,
}

impl SignerConfig {
    /// Creates a mnemonic signer for config (yaml format) store at given path
    pub async fn create_mnemonic_signer(path: impl AsRef<Path>) -> Result<MnemonicSigner> {
        Self::read(path).await?.get_mnemonic_signer()
    }

    /// Adds signer config to given mnemonic signer
    fn add_to_signer(&self, mnemonic_signer: &mut MnemonicSigner) -> Result<()> {
        for chain_signer in self.chains.iter() {
            chain_signer.add_to_signer(mnemonic_signer)?;
        }

        Ok(())
    }

    /// Reads the signer config from given yaml file
    async fn read(path: impl AsRef<Path>) -> Result<Self> {
        let bytes = tokio::fs::read(path)
            .await
            .context("failed to read signer config")?;

        serde_yaml::from_slice(&bytes).context("failed to parse signer config")
    }

    /// Gets the mnemonic signer for current signer config
    fn get_mnemonic_signer(&self) -> Result<MnemonicSigner> {
        let mut mnemonic_signer = MnemonicSigner::new();
        self.add_to_signer(&mut mnemonic_signer)?;

        Ok(mnemonic_signer)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChainSignerConfig {
    /// Chain ID
    pub chain_id: ChainId,
    /// Mnemonic phrase of signer
    pub mnemonic: String,
    /// HD path of signer
    pub hd_path: Option<String>,
    /// Bech32 account prefix of signer
    pub account_prefix: Option<String>,
    /// Algorithm used for address generation [possible values: `secp256k1`, `eth-secp256k1`]
    pub algo: Option<String>,
}

impl ChainSignerConfig {
    /// Adds chain signer config to given mnemonic signer
    fn add_to_signer(&self, mnemonic_signer: &mut MnemonicSigner) -> Result<()> {
        mnemonic_signer
            .add_chain_config(
                self.chain_id.clone(),
                &self.mnemonic,
                self.hd_path.as_deref(),
                self.account_prefix.as_deref(),
                self.algo
                    .as_deref()
                    .map(FromStr::from_str)
                    .transpose()
                    .context(format!("invalid algo for chain id: {}", self.chain_id))?,
            )
            .context(format!(
                "invalid signer config for chain id: {}",
                self.chain_id
            ))?;

        Ok(())
    }
}
