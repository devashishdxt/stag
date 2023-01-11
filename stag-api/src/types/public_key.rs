use std::{fmt, str::FromStr};

use anyhow::{anyhow, Context, Error, Result};
use bech32::{ToBase32, Variant};
use k256::ecdsa::VerifyingKey;
#[cfg(feature = "ethermint")]
use k256::elliptic_curve::sec1::ToEncodedPoint;
use prost::Message;
use prost_types::Any;
use ripemd::Ripemd160;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
#[cfg(feature = "ethermint")]
use sha3::Keccak256;

#[cfg(feature = "ethermint")]
use crate::types::cosmos::crypto::{EthSecp256k1PubKey, ETH_SECP256K1_PUB_KEY_TYPE_URL};
use crate::types::{
    cosmos::crypto::{Secp256k1PubKey, SECP256K1_PUB_KEY_TYPE_URL},
    proto_util::AnyConvert,
};

use super::cosmos::crypto::{from_verifying_key, into_verifying_key};

/// Supported algorithms for address generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublicKeyAlgo {
    /// Secp256k1 (tendermint)
    Secp256k1,
    #[cfg(feature = "ethermint")]
    /// EthSecp256k1 (ethermint)
    EthSecp256k1,
}

impl fmt::Display for PublicKeyAlgo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Secp256k1 => write!(f, "secp256k1"),
            #[cfg(feature = "ethermint")]
            Self::EthSecp256k1 => write!(f, "eth-secp256k1"),
        }
    }
}

impl FromStr for PublicKeyAlgo {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "secp256k1" => Ok(Self::Secp256k1),
            #[cfg(feature = "ethermint")]
            "eth-secp256k1" => Ok(Self::EthSecp256k1),
            _ => Err(anyhow!("invalid address generation algorithm: {}", s)),
        }
    }
}

/// Public Key
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "public_key")]
pub enum PublicKey {
    /// Secp256k1 (tendermint)
    Secp256k1(VerifyingKey),
    #[cfg(feature = "ethermint")]
    /// EthSecp256k1 (ethermint)
    EthSecp256k1(VerifyingKey),
}

impl PublicKey {
    pub fn new(public_key: String, algo: PublicKeyAlgo) -> Result<Self> {
        let public_key_bytes =
            hex::decode(public_key).context("unable to decode new public key")?;

        let verifying_key = VerifyingKey::from_sec1_bytes(&public_key_bytes)
            .context("failed to decode sec1 bytes of new public key")?;

        match algo {
            PublicKeyAlgo::Secp256k1 => Ok(Self::Secp256k1(verifying_key)),
            #[cfg(feature = "ethermint")]
            PublicKeyAlgo::EthSecp256k1 => Ok(Self::EthSecp256k1(verifying_key)),
        }
    }

    /// Creates a new instance of PublicKey from a verifying key (using secp256k1)
    pub fn new_secp256k1(key: VerifyingKey) -> Self {
        Self::Secp256k1(key)
    }

    #[cfg(feature = "ethermint")]
    /// Creates a new instance of PublicKey from a verifying key (using eth-secp256k1)
    pub fn new_eth_secp256k1(key: VerifyingKey) -> Self {
        Self::EthSecp256k1(key)
    }

    /// Returns the public key algorithm
    pub fn algo(&self) -> PublicKeyAlgo {
        match self {
            Self::Secp256k1(_) => PublicKeyAlgo::Secp256k1,
            #[cfg(feature = "ethermint")]
            Self::EthSecp256k1(_) => PublicKeyAlgo::EthSecp256k1,
        }
    }

    /// Returns the address of the public key
    pub fn address(&self) -> Result<String> {
        Ok(hex::encode(self.address_bytes()?))
    }

    /// Returns the account address of the public key
    pub fn account_address(&self, prefix: &str) -> Result<String> {
        bech32::encode(prefix, self.address_bytes()?.to_base32(), Variant::Bech32)
            .map_err(Into::into)
    }

    fn address_bytes(&self) -> Result<Vec<u8>> {
        match self {
            #[cfg(feature = "ethermint")]
            Self::EthSecp256k1(ref key) => {
                use k256::EncodedPoint;

                let encoded_point: EncodedPoint = key.to_encoded_point(false);
                let hash = Keccak256::digest(&encoded_point.as_bytes()[1..])[12..].to_vec();

                Ok(hash)
            }
            Self::Secp256k1(ref key) => {
                Ok(Ripemd160::digest(Sha256::digest(key.to_bytes())).to_vec())
            }
        }
    }
}

impl AsRef<VerifyingKey> for PublicKey {
    fn as_ref(&self) -> &VerifyingKey {
        match self {
            Self::Secp256k1(key) => key,
            #[cfg(feature = "ethermint")]
            Self::EthSecp256k1(key) => key,
        }
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode_upper(self.as_ref().to_bytes()))
    }
}

impl AnyConvert for PublicKey {
    fn from_any(value: &Any) -> Result<Self> {
        match value.type_url.as_str() {
            #[cfg(feature = "ethermint")]
            ETH_SECP256K1_PUB_KEY_TYPE_URL => {
                let public_key: EthSecp256k1PubKey =
                    EthSecp256k1PubKey::decode(value.value.as_slice())?;
                Ok(Self::EthSecp256k1(TryFrom::try_from(&public_key)?))
            }
            SECP256K1_PUB_KEY_TYPE_URL => {
                let public_key: Secp256k1PubKey = Secp256k1PubKey::decode(value.value.as_slice())?;
                Ok(Self::Secp256k1(into_verifying_key(&public_key)?))
            }
            other => Err(anyhow!("unknown type url for `Any` type: `{}`", other)),
        }
    }

    fn to_any(&self) -> Result<Any> {
        match self {
            #[cfg(feature = "ethermint")]
            Self::EthSecp256k1(ref key) => {
                let public_key: EthSecp256k1PubKey = key.into();
                public_key.to_any()
            }
            Self::Secp256k1(ref key) => {
                let public_key: Secp256k1PubKey = from_verifying_key(key);
                public_key.to_any()
            }
        }
    }
}

// fn serialize_verifying_key<S>(key: &VerifyingKey, serializer: S) -> Result<S::Ok, S::Error>
// where
//     S: Serializer,
// {
//     hex::serialize_upper(key.to_bytes(), serializer)
// }

// fn deserialize_verifying_key<'de, D>(deserializer: D) -> Result<VerifyingKey, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     let bytes: Vec<u8> = hex::deserialize(deserializer)?;
//     VerifyingKey::from_sec1_bytes(&bytes).map_err(serde::de::Error::custom)
// }
