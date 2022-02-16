mod keplr;

use anyhow::{Context, Result};
use async_trait::async_trait;

use crate::types::{ics::core::ics24_host::identifier::ChainId, public_key::PublicKey};

use self::keplr::Keplr;

use super::{GetPublicKey, Message, Signer};

pub struct KeplrSigner {
    keplr: Keplr,
}

impl KeplrSigner {
    pub fn new() -> Result<Self> {
        Ok(Self {
            keplr: Keplr::get().context("keplr wallet is not available")?,
        })
    }
}

#[cfg_attr(feature = "non-wasm", async_trait)]
#[cfg_attr(feature = "wasm", async_trait(?Send))]
impl GetPublicKey for KeplrSigner {
    async fn get_public_key(&self, chain_id: &ChainId) -> Result<PublicKey> {
        self.keplr.get_public_key(chain_id).await
    }

    async fn to_account_address(&self, chain_id: &ChainId) -> Result<String> {
        self.keplr.to_account_address(chain_id).await
    }
}

#[cfg_attr(feature = "non-wasm", async_trait)]
#[cfg_attr(feature = "wasm", async_trait(?Send))]
impl Signer for KeplrSigner {
    async fn sign(
        &self,
        _request_id: Option<&str>,
        chain_id: &ChainId,
        message: Message<'_>,
    ) -> Result<Vec<u8>> {
        self.keplr.sign(chain_id, message.as_ref()).await
    }
}
