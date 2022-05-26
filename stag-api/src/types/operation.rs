use chrono::{DateTime, Utc};
use primitive_types::U256;
use serde::{Deserialize, Serialize};

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
pub enum OperationType {
    /// Mint some tokens on IBC enabled chain
    Mint {
        /// Address of the account
        to: String,
        /// Denom of tokens
        denom: Identifier,
        /// Amount of tokens
        amount: U256,
    },
    /// Burn some tokens on IBC enabled chain
    Burn {
        /// Address of the account
        from: String,
        /// Denom of tokens
        denom: Identifier,
        /// Amount of tokens
        amount: U256,
    },
}
