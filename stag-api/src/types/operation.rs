use primitive_types::U256;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

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
    pub created_at: OffsetDateTime,
}

/// Different types of possible operations on an account
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OperationType {
    /// Mint some tokens on IBC enabled chain
    Mint,
    /// Burn some tokens on IBC enabled chain
    Burn,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationRequest<'a> {
    /// Request ID for tracking purposes
    pub request_id: Option<&'a str>,
    /// Chain ID of the operation
    pub chain_id: &'a ChainId,
    /// Address of the account
    pub address: &'a str,
    /// Denom of tokens
    pub denom: &'a Identifier,
    /// Amount of tokens
    pub amount: &'a U256,
    /// Type of operation
    pub operation_type: OperationType,
    /// On-chain transaction hash (in hex)
    pub transaction_hash: &'a str,
    /// Time at which this operation was created
    pub created_at: OffsetDateTime,
}
