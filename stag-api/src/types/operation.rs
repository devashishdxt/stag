use core::fmt;
use std::str::FromStr;

use anyhow::{anyhow, Error};
use chrono::{DateTime, Utc};
use primitive_types::U256;
use serde::{Deserialize, Serialize};

use crate::types::ics::core::ics24_host::identifier::{ChainId, Identifier};

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
    /// Address of the account
    pub address: String,
    /// Denom of tokens
    pub denom: Identifier,
    /// Amount of tokens
    pub amount: U256,
    /// Type of operation
    pub operation_type: OperationType,
    /// On-chain transaction hash (in hex)
    pub transaction_hash: String,
    /// Time at which this operation was created
    pub created_at: DateTime<Utc>,
}

/// Different types of possible operations on an account
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationType {
    /// Mint some tokens on IBC enabled chain
    Mint,
    /// Burn some tokens on IBC enabled chain
    Burn,
}

impl fmt::Display for OperationType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OperationType::Mint => write!(f, "mint"),
            OperationType::Burn => write!(f, "burn"),
        }
    }
}

impl FromStr for OperationType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mint" => Ok(OperationType::Mint),
            "burn" => Ok(OperationType::Burn),
            _ => Err(anyhow!("invalid operation type: {}", s)),
        }
    }
}
