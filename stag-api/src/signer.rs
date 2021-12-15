use anyhow::Result;
use async_trait::async_trait;

use crate::{ChainId, PublicKey};

/// This trait must be implemented by all the public key providers (e.g. mnemonic, ledger, etc.)
pub trait GetPublicKey {
    /// Returns public key of signer
    fn get_public_key(&self, chain_id: &ChainId) -> Result<PublicKey>;

    /// Returns account prefix for computing bech32 addresses
    fn get_account_prefix(&self, chain_id: &ChainId) -> &str;

    /// Returns accounts address for this signer for given prefix
    fn to_account_address(&self, chain_id: &ChainId) -> Result<String>;
}

/// Type of message given to a signer
#[derive(Debug)]
pub enum Message<'a> {
    /// [cosmos_sdk_proto::ibc::lightclients::solomachine::v1::SignBytes]
    SignBytes(&'a [u8]),
    /// [cosmos_sdk_proto::cosmos::tx::v1beta1::SignDoc]
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
#[async_trait]
pub trait Signer: GetPublicKey + Send + Sync {
    /// Signs the given message
    async fn sign(
        &self,
        request_id: Option<&str>,
        chain_id: &ChainId,
        message: Message<'_>,
    ) -> Result<Vec<u8>>;
}
