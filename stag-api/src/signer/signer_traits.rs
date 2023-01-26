use anyhow::Result;
use async_trait::async_trait;

use crate::{
    trait_util::Base,
    types::{ics::core::ics24_host::identifier::ChainId, public_key::PublicKey},
};

/// This trait must be implemented by all the public key providers (e.g. mnemonic, ledger, etc.)
#[cfg_attr(all(not(feature = "wasm"), feature = "non-wasm"), async_trait)]
#[cfg_attr(feature = "wasm", async_trait(?Send))]
pub trait GetPublicKey: Base {
    /// Returns public key of signer
    async fn get_public_key(&self, chain_id: &ChainId) -> Result<PublicKey>;

    /// Returns accounts address for this signer for given prefix
    async fn to_account_address(&self, chain_id: &ChainId) -> Result<String>;
}

/// Type of message given to a signer
#[derive(Debug)]
pub enum Message<'a> {
    /// [cosmos_sdk_proto::ibc::lightclients::solomachine::v2::SignBytes] or [crate::types::proto::ibc::lightclients::solomachine::v3::SignBytes]
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
#[cfg_attr(all(not(feature = "wasm"), feature = "non-wasm"), async_trait)]
#[cfg_attr(feature = "wasm", async_trait(?Send))]
pub trait Signer: GetPublicKey {
    /// Signs the given message
    async fn sign(
        &self,
        request_id: Option<&str>,
        chain_id: &ChainId,
        message: Message<'_>,
    ) -> Result<Vec<u8>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_message_type() {
        let message = Message::SignBytes(b"test");
        assert_eq!(message.message_type(), "sign-bytes");

        let message = Message::SignDoc(b"test");
        assert_eq!(message.message_type(), "sign-doc");
    }

    #[tokio::test]
    async fn test_message_as_ref() {
        let message = Message::SignBytes(b"test");
        assert_eq!(message.as_ref(), b"test");

        let message = Message::SignDoc(b"test");
        assert_eq!(message.as_ref(), b"test");
    }
}
