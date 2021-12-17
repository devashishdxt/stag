mod builder;

pub use self::builder::SignerConfig;

#[cfg(feature = "mnemonic-signer")]
pub use self::builder::MnemonicSigner;

#[cfg(feature = "mnemonic-signer")]
mod mnemonic_signer;

use anyhow::Result;
use async_trait::async_trait;

use crate::{ChainId, PublicKey};

/// This trait must be implemented by all the public key providers (e.g. mnemonic, ledger, etc.)
pub trait GetPublicKey {
    /// Returns public key of signer
    fn get_public_key(&self, chain_id: &ChainId) -> Result<PublicKey>;

    /// Returns account prefix for computing bech32 addresses
    fn get_account_prefix(&self, chain_id: &ChainId) -> Result<String>;

    /// Returns accounts address for this signer for given prefix
    fn to_account_address(&self, chain_id: &ChainId) -> Result<String>;
}

impl<T> GetPublicKey for &T
where
    T: GetPublicKey,
{
    fn get_public_key(&self, chain_id: &ChainId) -> Result<PublicKey> {
        (*self).get_public_key(chain_id)
    }

    fn get_account_prefix(&self, chain_id: &ChainId) -> Result<String> {
        (*self).get_account_prefix(chain_id)
    }

    fn to_account_address(&self, chain_id: &ChainId) -> Result<String> {
        (*self).to_account_address(chain_id)
    }
}

/// Type of message given to a signer
#[derive(Debug)]
pub enum Message<'a> {
    /// [crate::types::proto::ibc::lightclients::solomachine::v1::SignBytes]
    SignBytes(&'a [u8]),
    /// [crate::types::proto::cosmos::tx::v1beta1::SignDoc]
    SignDoc(&'a [u8]),
}

impl<'a> Message<'a> {
    /// Returns the message type of current message
    pub fn message_type(&self) -> &'static str {
        match self {
            Self::SignBytes(_) => "sign-bytes",
            Self::SignDoc(_) => "sign-doc",
        }
    }
}

impl AsRef<[u8]> for Message<'_> {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::SignBytes(bytes) => bytes,
            Self::SignDoc(bytes) => bytes,
        }
    }
}

/// This trait must be implemented by all the transaction signers (e.g. mnemonic, ledger, etc.)
#[cfg_attr(not(feature = "wasm"), async_trait)]
#[cfg_attr(feature = "wasm", async_trait(?Send))]
pub trait Signer: GetPublicKey + Send + Sync {
    /// Signs the given message
    async fn sign(
        &self,
        request_id: Option<&str>,
        chain_id: &ChainId,
        message: Message<'_>,
    ) -> Result<Vec<u8>>;
}
