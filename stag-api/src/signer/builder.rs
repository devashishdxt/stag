#[cfg(feature = "mnemonic-signer")]
use std::collections::HashMap;

use anyhow::{ensure, Result};
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
        &mut self,
        chain_id: ChainId,
        mnemonic: &str,
        hd_path: Option<&str>,
        account_prefix: Option<&str>,
        algo: Option<PublicKeyAlgo>,
    ) -> Result<&mut Self> {
        ensure!(
            !self.config_map.contains_key(&chain_id),
            "Signer config for chain id {} already exists",
            chain_id
        );

        self.config_map.insert(
            chain_id,
            MnemonicSignerConfig::new(mnemonic, hd_path, account_prefix, algo)?,
        );

        Ok(self)
    }

    /// Updates configuration for a chain id to the mnemonic signer
    pub fn update_chain_config(
        &mut self,
        chain_id: ChainId,
        mnemonic: &str,
        hd_path: Option<&str>,
        account_prefix: Option<&str>,
        algo: Option<PublicKeyAlgo>,
    ) -> Result<&mut Self> {
        ensure!(
            self.config_map.contains_key(&chain_id),
            "Signer config for chain id {} does not exist",
            chain_id
        );

        self.config_map.insert(
            chain_id,
            MnemonicSignerConfig::new(mnemonic, hd_path, account_prefix, algo)?,
        );

        Ok(self)
    }

    /// Checks if mnemonic signer has configuration for a chain id
    pub fn has_chain_config(&self, chain_id: &ChainId) -> bool {
        self.config_map.contains_key(chain_id)
    }

    /// Returns a list of chain IDs and their corresponding account addresses
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

#[cfg(test)]
mod tests {
    use crate::signer::GetPublicKey;

    use super::*;

    const MNEMONIC: &str = "practice empty client sauce pistol work ticket casual romance appear army fault palace coyote fox super salute slim catch kite wrist three hedgehog sign";
    const ACCOUNT_ADDRESS: &str = "cosmos1j2qpprh2xke7qjqzehfqgjdkfgddf9dm06dugw";

    #[test]
    fn test_signer_config() {
        let config = MnemonicSigner::new();
        let signer = config.into_signer().unwrap();
        assert_eq!(signer, MnemonicSignerImpl::new(HashMap::new()));
    }

    #[tokio::test]
    async fn test_add_chain_config() {
        let chain_id: ChainId = "test-1".parse().unwrap();

        let mut config = MnemonicSigner::new();
        assert!(config
            .add_chain_config(chain_id.clone(), MNEMONIC, None, None, None)
            .is_ok());

        let signer = config.into_signer();
        assert!(signer.is_ok());
        let signer = signer.unwrap();

        let account_address = signer.to_account_address(&chain_id).await;
        assert!(account_address.is_ok());
        let account_address = account_address.unwrap();

        assert_eq!(account_address, ACCOUNT_ADDRESS);
    }

    #[test]
    fn test_add_invalid_chain_config() {
        let chain_id: ChainId = "test-1".parse().unwrap();

        let mut config = MnemonicSigner::new();
        assert!(config
            .add_chain_config(chain_id, "invalid mnemonic", None, None, None)
            .is_err());
    }

    #[test]
    fn test_get_signers() {
        let mut config = MnemonicSigner::new();
        assert!(config
            .add_chain_config("test-1".parse().unwrap(), MNEMONIC, None, None, None)
            .is_ok());

        let signers = config.get_signers().unwrap();
        assert_eq!(signers.len(), 1);
        assert_eq!(signers[0].0, "test-1".parse().unwrap());
        assert_eq!(signers[0].1, ACCOUNT_ADDRESS);
    }

    #[test]
    fn test_cannot_add_same_config_twice() {
        let chain_id: ChainId = "test-1".parse().unwrap();

        let mut config = MnemonicSigner::new();
        assert!(config
            .add_chain_config(chain_id.clone(), MNEMONIC, None, None, None)
            .is_ok());

        assert!(config
            .add_chain_config(chain_id, MNEMONIC, None, None, None)
            .is_err());
    }

    #[test]
    fn test_update_config() {
        let chain_id: ChainId = "test-1".parse().unwrap();

        let mut config = MnemonicSigner::new();
        assert!(config
            .add_chain_config(chain_id.clone(), MNEMONIC, None, None, None)
            .is_ok());

        assert!(config
            .update_chain_config(
                chain_id,
                MNEMONIC,
                Some("m/44'/394'/0'/0/0"),
                Some("cro"),
                None
            )
            .is_ok());
    }

    #[test]
    fn test_cannot_update_non_existing_config() {
        let chain_id: ChainId = "test-1".parse().unwrap();

        let mut config = MnemonicSigner::new();
        assert!(config
            .update_chain_config(chain_id, MNEMONIC, None, None, None)
            .is_err());
    }
}
