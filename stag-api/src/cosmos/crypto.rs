use std::convert::TryFrom;

use anyhow::{Context, Error};
use k256::ecdsa::VerifyingKey;

pub use crate::proto::cosmos::crypto::secp256k1::PubKey as Secp256k1PubKey;
#[cfg(feature = "ethermint")]
pub use crate::proto::ethermint::crypto::v1::ethsecp256k1::PubKey as EthSecp256k1PubKey;

pub const SECP256K1_PUB_KEY_TYPE_URL: &str = "/cosmos.crypto.secp256k1.PubKey";
#[cfg(feature = "ethermint")]
pub const ETH_SECP256K1_PUB_KEY_TYPE_URL: &str = "/ethermint.crypto.v1.ethsecp256k1.PubKey";

impl From<&VerifyingKey> for Secp256k1PubKey {
    fn from(key: &VerifyingKey) -> Self {
        Self {
            key: key.to_bytes().to_vec(),
        }
    }
}

impl TryFrom<&Secp256k1PubKey> for VerifyingKey {
    type Error = Error;

    fn try_from(value: &Secp256k1PubKey) -> Result<Self, Self::Error> {
        Self::from_sec1_bytes(&value.key).context("unable to parse verifying key from sec1 bytes")
    }
}

impl_any_conversion!(Secp256k1PubKey, SECP256K1_PUB_KEY_TYPE_URL);

#[cfg(feature = "ethermint")]
impl From<&VerifyingKey> for EthSecp256k1PubKey {
    fn from(key: &VerifyingKey) -> Self {
        Self {
            key: key.to_bytes().to_vec(),
        }
    }
}

#[cfg(feature = "ethermint")]
impl TryFrom<&EthSecp256k1PubKey> for VerifyingKey {
    type Error = Error;

    fn try_from(value: &EthSecp256k1PubKey) -> Result<Self, Self::Error> {
        Self::from_sec1_bytes(&value.key).context("unable to parse verifying key from sec1 bytes")
    }
}

#[cfg(feature = "ethermint")]
impl_any_conversion!(EthSecp256k1PubKey, ETH_SECP256K1_PUB_KEY_TYPE_URL);
