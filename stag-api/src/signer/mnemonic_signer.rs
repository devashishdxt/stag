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

#[derive(Clone)]
/// Signer implementation using mnemonic
pub struct MnemonicSigner {
    config_map: HashMap<ChainId, MnemonicSignerConfig>,
}

/// Configuration for mnemonic signer
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

impl MnemonicSignerConfig {
    /// Returns the signing key
    fn get_signing_key(&self) -> Result<SigningKey> {
        let seed = self.mnemonic.to_seed("");
        let hd_path = DerivationPath::from_str(&self.hd_path).context("invalid HD path")?;
        let private_key =
            ExtendedPrivateKey::<SigningKey>::derive_from_path(seed.as_bytes(), &hd_path).unwrap();

        Ok(private_key.into())
    }

    /// Returns the public key
    fn get_public_key(&self) -> Result<PublicKey> {
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

#[cfg_attr(not(feature = "wasm"), async_trait)]
#[cfg_attr(feature = "wasm", async_trait(?Send))]
impl GetPublicKey for MnemonicSigner {
    async fn get_public_key(&self, chain_id: &ChainId) -> Result<PublicKey> {
        self.get_config(chain_id)?.get_public_key()
    }

    async fn to_account_address(&self, chain_id: &ChainId) -> Result<String> {
        self.get_config(chain_id)?.get_account_address()
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
    use super::*;

    #[test]
    fn test_mnemonic_signer() {
        let config = MnemonicSignerConfig::new("practice empty client sauce pistol work ticket casual romance appear army fault palace coyote fox super salute slim catch kite wrist three hedgehog sign", None, None, None).unwrap();

        let signing_key = config.get_signing_key().unwrap();

        let signature: Signature = match config.algo {
            PublicKeyAlgo::Secp256k1 => <SigningKey as k256::ecdsa::signature::Signer<
                k256::ecdsa::Signature,
            >>::sign(&signing_key, &[1]),
            #[cfg(feature = "ethermint")]
            PublicKeyAlgo::EthSecp256k1 => <SigningKey as k256::ecdsa::signature::Signer<
                k256::ecdsa::recoverable::Signature,
            >>::sign(&signing_key, &[1])
            .into(),
        };

        assert_eq!(
            signature.as_ref().to_vec(),
            vec![
                111, 153, 170, 175, 174, 45, 141, 109, 219, 33, 166, 96, 118, 24, 252, 73, 189,
                237, 250, 246, 13, 174, 51, 44, 29, 164, 211, 55, 110, 155, 240, 84, 111, 147, 217,
                163, 5, 147, 155, 232, 251, 73, 25, 56, 119, 163, 76, 246, 77, 11, 100, 79, 174,
                230, 255, 51, 47, 231, 46, 133, 125, 247, 214, 202
            ]
        );

        // assert_eq!(
        //     account_address,
        //     "cosmos1qx0cppqkpwyyjvcl8p4s5eehrftfp00wdtkuyz"
        // );

        // let signature = "b5mqr64tjW3bIaZgdhj8Sb3t+vYNrjMsHaTTN26b8FRvk9mjBZOb6PtJGTh3o0z2TQtkT67m/zMv5y6FfffWyg==";
    }
}
