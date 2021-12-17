use std::{collections::HashMap, str::FromStr};

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use bip32::{DerivationPath, ExtendedPrivateKey, Language, Mnemonic};
use k256::ecdsa::{signature::DigestSigner, Signature, SigningKey};
use sha2::Digest;

use crate::{ChainId, PublicKey, PublicKeyAlgo};

use super::{GetPublicKey, Message, Signer};

const DEFAULT_HD_PATH: &str = "m/44'/118'/0'/0/0";
const DEFAULT_ACCOUNT_PREFIX: &str = "cosmos";
const DEFAULT_ADDRESS_ALGO: &str = "secp256k1";

#[derive(Clone)]
/// Signer implementation using mnemonic
pub struct MnemonicSigner {
    config_map: HashMap<ChainId, MnemonicSignerConfig>,
}

#[derive(Clone)]
pub struct MnemonicSignerConfig {
    /// Mnemonic of signer
    pub mnemonic: Mnemonic,
    /// HD path of signer
    pub hd_path: String,
    /// Bech32 prefix
    pub account_prefix: String,
    /// Algorithm used for address generation
    pub algo: PublicKeyAlgo,
}

impl MnemonicSignerConfig {
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
}

impl MnemonicSigner {
    pub fn new(config_map: HashMap<ChainId, MnemonicSignerConfig>) -> Self {
        Self { config_map }
    }

    fn get_config(&self, chain_id: &ChainId) -> Result<&MnemonicSignerConfig> {
        self.config_map
            .get(chain_id)
            .ok_or_else(|| anyhow!("no signer config for chain id: {}", chain_id))
    }
}

impl MnemonicSignerConfig {
    fn get_signing_key(&self) -> Result<SigningKey> {
        let seed = self.mnemonic.to_seed("");
        let hd_path = DerivationPath::from_str(&self.hd_path).context("invalid HD path")?;
        let private_key =
            ExtendedPrivateKey::<SigningKey>::derive_from_path(seed.as_bytes(), &hd_path).unwrap();

        Ok(private_key.into())
    }

    fn get_public_key(&self) -> Result<PublicKey> {
        let signing_key = self.get_signing_key()?;
        let verifying_key = signing_key.verifying_key();

        match self.algo {
            PublicKeyAlgo::Secp256k1 => Ok(PublicKey::Secp256k1(verifying_key)),
            #[cfg(feature = "ethermint")]
            PublicKeyAlgo::EthSecp256k1 => Ok(PublicKey::EthSecp256k1(verifying_key)),
        }
    }

    fn get_account_prefix(&self) -> &str {
        &self.account_prefix
    }

    fn to_account_address(&self) -> Result<String> {
        self.get_public_key()?
            .account_address(self.get_account_prefix())
    }
}

impl GetPublicKey for MnemonicSigner {
    fn get_public_key(&self, chain_id: &ChainId) -> Result<PublicKey> {
        self.get_config(chain_id)?.get_public_key()
    }

    fn get_account_prefix(&self, chain_id: &ChainId) -> Result<String> {
        Ok(self.get_config(chain_id)?.get_account_prefix().to_owned())
    }

    fn to_account_address(&self, chain_id: &ChainId) -> Result<String> {
        self.get_config(chain_id)?.to_account_address()
    }
}

#[cfg_attr(not(feature = "wasm"), async_trait)]
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
            PublicKeyAlgo::Secp256k1 => signing_key.sign_digest(sha2::Sha256::new().chain(message)),
            #[cfg(feature = "ethermint")]
            PublicKeyAlgo::EthSecp256k1 => {
                signing_key.sign_digest(sha3::Keccak256::new().chain(message))
            }
        };

        Ok(signature.as_ref().to_vec())
    }
}
