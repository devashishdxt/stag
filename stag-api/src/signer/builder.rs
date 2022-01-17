#[cfg(feature = "mnemonic-signer")]
use std::collections::HashMap;

use anyhow::Result;
use sealed::sealed;

use crate::trait_util::Base;
#[cfg(feature = "mnemonic-signer")]
use crate::types::{ics::core::ics24_host::identifier::ChainId, public_key::PublicKeyAlgo};

#[cfg(feature = "keplr-signer")]
use super::keplr_signer::KeplrSigner as KeplrSignerImpl;
#[cfg(feature = "mnemonic-signer")]
use super::mnemonic_signer::{MnemonicSigner as MnemonicSignerImpl, MnemonicSignerConfig};
use super::Signer;

/// Configuration for signer
#[sealed]
pub trait SignerConfig: Base {
    /// Concrete signer type that this config will produce
    type Signer: Signer;

    /// Create concrete signer from this config
    fn into_signer(self) -> Result<Self::Signer>;
}

#[cfg_attr(feature = "doc", doc(cfg(feature = "keplr-signer")))]
#[cfg(feature = "keplr-signer")]
#[derive(Default)]
/// Signer backend using keplr wallet
pub struct KeplrSigner;

#[cfg_attr(feature = "doc", doc(cfg(feature = "keplr-signer")))]
#[cfg(feature = "keplr-signer")]
#[sealed]
impl SignerConfig for KeplrSigner {
    type Signer = KeplrSignerImpl;

    fn into_signer(self) -> Result<Self::Signer> {
        KeplrSignerImpl::new()
    }
}

#[cfg_attr(feature = "doc", doc(cfg(feature = "mnemonic-signer")))]
#[cfg(feature = "mnemonic-signer")]
#[derive(Default, Clone, PartialEq, Eq)]
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

    pub fn get_signers(&self) -> Result<Vec<(ChainId, String)>> {
        self.config_map
            .iter()
            .map(|(chain_id, config)| -> Result<(ChainId, String)> {
                Ok((chain_id.clone(), config.get_account_address()?))
            })
            .collect()
    }
}

#[cfg_attr(feature = "doc", doc(cfg(feature = "mnemonic-signer")))]
#[cfg(feature = "mnemonic-signer")]
#[sealed]
impl SignerConfig for MnemonicSigner {
    type Signer = MnemonicSignerImpl;

    fn into_signer(self) -> Result<Self::Signer> {
        Ok(MnemonicSignerImpl::new(self.config_map))
    }
}
