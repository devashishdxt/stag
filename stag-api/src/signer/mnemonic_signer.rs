use std::{collections::HashMap, str::FromStr};

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use bip32::{DerivationPath, ExtendedPrivateKey, Language, Mnemonic};
use k256::ecdsa::{Signature, SigningKey};

use crate::types::{
    ics::core::ics24_host::identifier::ChainId,
    public_key::{PublicKey, PublicKeyAlgo},
};

use super::{GetPublicKey, Message, Signer};

const DEFAULT_HD_PATH: &str = "m/44'/118'/0'/0/0";
const DEFAULT_ACCOUNT_PREFIX: &str = "cosmos";
const DEFAULT_ADDRESS_ALGO: &str = "secp256k1";

#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
#[derive(Clone)]
/// Signer implementation using mnemonic
pub struct MnemonicSigner {
    config_map: HashMap<ChainId, MnemonicSignerConfig>,
}

/// Configuration for mnemonic signer
#[derive(Clone)]
pub struct MnemonicSignerConfig {
    /// Mnemonic of signer
    mnemonic: Mnemonic,
    /// HD path of signer
    hd_path: String,
    /// Bech32 prefix
    account_prefix: String,
    /// Algorithm used for address generation
    algo: PublicKeyAlgo,
}

impl PartialEq for MnemonicSignerConfig {
    fn eq(&self, other: &Self) -> bool {
        self.mnemonic.phrase() == other.mnemonic.phrase()
            && self.hd_path == other.hd_path
            && self.account_prefix == other.account_prefix
            && self.algo == other.algo
    }
}

impl Eq for MnemonicSignerConfig {}

impl MnemonicSignerConfig {
    /// Creates a new instance of mnemonic signer configuration
    pub fn new(
        mnemonic: &str,
        hd_path: Option<&str>,
        account_prefix: Option<&str>,
        algo: Option<PublicKeyAlgo>,
    ) -> Result<Self> {
        let mnemonic =
            Mnemonic::new(mnemonic, Language::English).map_err(|_| anyhow!("invalid mnemonic"))?;

        let hd_path = hd_path.unwrap_or(DEFAULT_HD_PATH).to_string();
        let account_prefix = account_prefix.unwrap_or(DEFAULT_ACCOUNT_PREFIX).to_string();

        let algo = algo.unwrap_or_else(|| DEFAULT_ADDRESS_ALGO.parse().unwrap());

        Ok(MnemonicSignerConfig {
            mnemonic,
            hd_path,
            account_prefix,
            algo,
        })
    }

    /// Returns the signing key
    fn get_signing_key(&self) -> Result<SigningKey> {
        let seed = self.mnemonic.to_seed("");
        let hd_path = DerivationPath::from_str(&self.hd_path).context("invalid HD path")?;
        let private_key =
            ExtendedPrivateKey::<SigningKey>::derive_from_path(seed.as_bytes(), &hd_path).unwrap();

        Ok(private_key.into())
    }

    /// Returns the public key
    pub(crate) fn get_public_key(&self) -> Result<PublicKey> {
        let signing_key = self.get_signing_key()?;
        let verifying_key = signing_key.verifying_key();

        match self.algo {
            PublicKeyAlgo::Secp256k1 => Ok(PublicKey::Secp256k1(verifying_key)),
            #[cfg(feature = "ethermint")]
            PublicKeyAlgo::EthSecp256k1 => Ok(PublicKey::EthSecp256k1(verifying_key)),
        }
    }

    /// Returns account address prefix
    fn get_account_prefix(&self) -> &str {
        &self.account_prefix
    }

    /// Returns the account address
    pub(crate) fn get_account_address(&self) -> Result<String> {
        self.get_public_key()?
            .account_address(self.get_account_prefix())
    }
}

impl MnemonicSigner {
    /// Creates a new instance of mnemonic signer
    pub fn new(config_map: HashMap<ChainId, MnemonicSignerConfig>) -> Self {
        Self { config_map }
    }

    /// Returns configuration for a chain id
    fn get_config(&self, chain_id: &ChainId) -> Result<&MnemonicSignerConfig> {
        self.config_map
            .get(chain_id)
            .ok_or_else(|| anyhow!("no signer config for chain id: {}", chain_id))
    }
}

#[cfg_attr(all(not(feature = "wasm"), feature = "non-wasm"), async_trait)]
#[cfg_attr(feature = "wasm", async_trait(?Send))]
impl GetPublicKey for MnemonicSigner {
    async fn get_public_key(&self, chain_id: &ChainId) -> Result<PublicKey> {
        self.get_config(chain_id)?.get_public_key()
    }

    async fn to_account_address(&self, chain_id: &ChainId) -> Result<String> {
        self.get_config(chain_id)?.get_account_address()
    }
}

#[cfg_attr(all(not(feature = "wasm"), feature = "non-wasm"), async_trait)]
#[cfg_attr(feature = "wasm", async_trait(?Send))]
impl Signer for MnemonicSigner {
    async fn sign(
        &self,
        _request_id: Option<&str>,
        chain_id: &ChainId,
        message: Message<'_>,
    ) -> Result<Vec<u8>> {
        let config = self.get_config(chain_id)?;

        let signing_key = config.get_signing_key()?;

        let signature: Signature = match config.algo {
            PublicKeyAlgo::Secp256k1 => <SigningKey as k256::ecdsa::signature::Signer<
                k256::ecdsa::Signature,
            >>::sign(&signing_key, message.as_ref()),
            #[cfg(feature = "ethermint")]
            PublicKeyAlgo::EthSecp256k1 => <SigningKey as k256::ecdsa::signature::Signer<
                k256::ecdsa::recoverable::Signature,
            >>::sign(&signing_key, message.as_ref())
            .into(),
        };

        Ok(signature.as_ref().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use std::fmt;

    use super::*;

    const MNEMONIC: &str = "practice empty client sauce pistol work ticket casual romance appear army fault palace coyote fox super salute slim catch kite wrist three hedgehog sign";
    const ACCOUNT_ADDRESS: &str = "cosmos1j2qpprh2xke7qjqzehfqgjdkfgddf9dm06dugw";

    impl fmt::Debug for MnemonicSignerConfig {
        #[cfg_attr(coverage, no_coverage)]
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("MnemonicSignerConfig")
                .field("mnemonic", &self.mnemonic.phrase())
                .field("hd_path", &self.hd_path)
                .field("account_prefix", &self.account_prefix)
                .field("algo", &self.algo)
                .finish()
        }
    }

    #[test]
    fn test_mnemonic_signer_config() {
        let config = MnemonicSignerConfig::new(MNEMONIC, None, None, None);
        assert!(config.is_ok());
        let config = config.unwrap();

        assert_eq!(
            config,
            MnemonicSignerConfig::new(
                MNEMONIC,
                Some(DEFAULT_HD_PATH),
                Some(DEFAULT_ACCOUNT_PREFIX),
                Some(PublicKeyAlgo::Secp256k1),
            )
            .unwrap()
        );
    }

    #[tokio::test]
    async fn test_mnemonic_signer() {
        // Chain id
        let chain_id: ChainId = "test-1".parse().unwrap();

        // Prepare signer
        let config = MnemonicSignerConfig::new(MNEMONIC, None, None, None);
        assert!(config.is_ok());
        let config = config.unwrap();

        let mut config_map = HashMap::with_capacity(1);
        config_map.insert(chain_id.clone(), config);

        let signer = MnemonicSigner::new(config_map);

        // Signer should not return public key with invalid chain id
        assert!(signer
            .get_public_key(&"test-2".parse().unwrap())
            .await
            .is_err());

        // Signer should return public key with valid chain id
        let public_key = signer.get_public_key(&chain_id).await;
        assert!(public_key.is_ok());
        let public_key = public_key.unwrap();

        let account_address = public_key.account_address("cosmos");
        assert!(account_address.is_ok());
        let account_address = account_address.unwrap();

        assert_eq!(account_address, ACCOUNT_ADDRESS);

        // Signer should not return account address with invalid chain id
        assert!(signer
            .to_account_address(&"test-2".parse().unwrap())
            .await
            .is_err());

        // Signer should return account address with valid chain id
        let account_address = signer.to_account_address(&chain_id).await;
        assert!(account_address.is_ok());
        let account_address = account_address.unwrap();

        assert_eq!(account_address, ACCOUNT_ADDRESS);

        // Signer should not sign message with invalid chain id
        assert!(signer
            .sign(
                None,
                &"test-2".parse().unwrap(),
                Message::SignBytes(b"test-message"),
            )
            .await
            .is_err());

        // Signer should sign message with valid chain id
        let signature = signer
            .sign(None, &chain_id, Message::SignBytes(b"test-message"))
            .await;
        assert!(signature.is_ok());
        let signature = signature.unwrap();

        assert_eq!(
            signature,
            vec![
                71, 82, 197, 57, 152, 49, 47, 42, 6, 226, 37, 53, 124, 193, 12, 157, 148, 219, 145,
                144, 226, 209, 97, 85, 136, 234, 249, 4, 41, 130, 141, 216, 40, 235, 22, 51, 7,
                195, 153, 188, 51, 147, 146, 159, 167, 147, 131, 74, 51, 216, 65, 152, 71, 130,
                239, 221, 110, 246, 155, 68, 159, 21, 168, 138
            ]
        );
    }
}
