use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::types::{
    ics::core::ics24_host::identifier::{ChainId, PortId},
    operation::OperationType,
};

/// Signer's public key entry for an IBC enabled chain
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainKeyRequest<'a> {
    /// Chain ID
    pub chain_id: &'a ChainId,
    /// Public key of signer
    pub public_key: &'a str,
    /// Creation time of chain key entry
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationRequest<'a> {
    /// Request ID for tracking purposes
    pub request_id: Option<&'a str>,
    /// Chain ID of the operation
    pub chain_id: &'a ChainId,
    /// Port ID of the channel
    pub port_id: &'a PortId,
    /// Type of operation
    pub operation_type: &'a OperationType,
    /// On-chain transaction hash (in hex)
    pub transaction_hash: &'a str,
    /// Time at which this operation was created
    pub created_at: DateTime<Utc>,
}
