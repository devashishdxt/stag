use chrono::{DateTime, Utc};
use primitive_types::U256;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::types::ics::core::ics24_host::identifier::{ChainId, Identifier};

use super::ics::core::ics24_host::identifier::PortId;

/// Denotes an operation on an account
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Operation {
    /// ID of operation
    pub id: i64,
    /// Request ID for tracking purposes
    pub request_id: Option<String>,
    /// Chain ID of the operation
    pub chain_id: ChainId,
    /// Port ID of the channel
    pub port_id: PortId,
    /// Type of operation
    pub operation_type: OperationType,
    /// On-chain transaction hash (in hex)
    pub transaction_hash: String,
    /// Time at which this operation was created
    pub created_at: DateTime<Utc>,
}

/// Different types of possible operations on an account
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum OperationType {
    /// Mint some tokens on IBC enabled chain
    #[serde(rename_all = "camelCase")]
    Mint {
        /// Address of the account
        to: String,
        /// Denom of tokens
        denom: Identifier,
        /// Amount of tokens
        #[serde(
            serialize_with = "serialize_u256",
            deserialize_with = "deserialize_u256"
        )]
        amount: U256,
    },
    /// Burn some tokens on IBC enabled chain
    #[serde(rename_all = "camelCase")]
    Burn {
        /// Address of the account
        from: String,
        /// Denom of tokens
        denom: Identifier,
        /// Amount of tokens
        #[serde(
            serialize_with = "serialize_u256",
            deserialize_with = "deserialize_u256"
        )]
        amount: U256,
    },
    /// Send some tokens from ICA account on host chain
    #[serde(rename_all = "camelCase")]
    IcaSend {
        /// Address of the account
        to: String,
        /// Denom of tokens
        denom: Identifier,
        /// Amount of tokens
        #[serde(
            serialize_with = "serialize_u256",
            deserialize_with = "deserialize_u256"
        )]
        amount: U256,
    },
    /// Delegate some tokens from ICA account on host chain to validator address
    #[serde(rename_all = "camelCase")]
    IcaDelegate {
        /// Address of the validator
        validator_address: String,
        /// Denom of tokens
        denom: Identifier,
        /// Amount of tokens
        #[serde(
            serialize_with = "serialize_u256",
            deserialize_with = "deserialize_u256"
        )]
        amount: U256,
    },
    /// Undelegate some tokens to ICA account on host chain from validator address
    #[serde(rename_all = "camelCase")]
    IcaUndelegate {
        /// Address of the validator
        validator_address: String,
        /// Denom of tokens
        denom: Identifier,
        /// Amount of tokens
        #[serde(
            serialize_with = "serialize_u256",
            deserialize_with = "deserialize_u256"
        )]
        amount: U256,
    },
}

fn serialize_u256<S>(value: &U256, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&value.to_string())
}

fn deserialize_u256<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    U256::from_dec_str(&s).map_err(serde::de::Error::custom)
}
