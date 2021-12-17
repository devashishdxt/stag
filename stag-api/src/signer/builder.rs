#[cfg(feature = "mnemonic-signer")]
use std::collections::HashMap;

#[cfg(feature = "mnemonic-signer")]
use anyhow::Result;
use sealed::sealed;

#[cfg(feature = "mnemonic-signer")]
use crate::{ChainId, PublicKeyAlgo};

#[cfg(feature = "mnemonic-signer")]
use super::mnemonic_signer::{MnemonicSigner as MnemonicSignerImpl, MnemonicSignerConfig};
use super::Signer;

#[sealed]
pub trait SignerConfig {
    type Signer: Signer;

    fn into_signer(self) -> Self::Signer;
}

#[cfg(feature = "mnemonic-signer")]
#[derive(Default)]
/// Signer backend using mnemonic
pub struct MnemonicSigner {
    config_map: HashMap<ChainId, MnemonicSignerConfig>,
}

#[cfg(feature = "mnemonic-signer")]
impl MnemonicSigner {
    /// Creates a new instance of mnemonic signer
    pub fn new() -> Self {
        Default::default()
    }

    /// Adds configuration for a chain id to the mnemonic signer
    pub fn add_chain_config(
        mut self,
        chain_id: ChainId,
        mnemonic: &str,
        hd_path: Option<&str>,
        account_prefix: Option<&str>,
        algo: Option<PublicKeyAlgo>,
    ) -> Result<Self> {
        self.config_map.insert(
            chain_id,
            MnemonicSignerConfig::new(mnemonic, hd_path, account_prefix, algo)?,
        );

        Ok(self)
    }
}

#[cfg(feature = "mnemonic-signer")]
#[sealed]
impl SignerConfig for MnemonicSigner {
    type Signer = MnemonicSignerImpl;

    fn into_signer(self) -> Self::Signer {
        MnemonicSignerImpl::new(self.config_map)
    }
}
