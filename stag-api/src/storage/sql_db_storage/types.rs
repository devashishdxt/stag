use anyhow::{ensure, Error};
use chrono::{DateTime, Utc};
use sqlx::{types::Json, Row};

use crate::types::{
    chain_state::{ChainConfig, ChainKey, ChainState, ConnectionDetails},
    ibc_data::IbcData,
    operation::Operation,
};

use super::DbRow;

impl TryFrom<DbRow> for Operation {
    type Error = Error;

    fn try_from(row: DbRow) -> Result<Self, Self::Error> {
        let id: i64 = row.try_get("id")?;
        let request_id: Option<String> = row.try_get("request_id")?;
        let chain_id: String = row.try_get("chain_id")?;
        let address: String = row.try_get("address")?;
        let denom: String = row.try_get("denom")?;
        let amount: Vec<u8> = row.try_get("amount")?;
        let operation_type: String = row.try_get("operation_type")?;
        let transaction_hash: String = row.try_get("transaction_hash")?;
        let created_at: DateTime<Utc> = row.try_get("created_at")?;

        let mut amount_bytes = [0; 32];

        ensure!(
            amount.len() == 32,
            "expected amount in u256 little endian bytes {}",
            amount.len()
        );

        amount_bytes.copy_from_slice(&amount);

        Ok(Self {
            id,
            request_id,
            chain_id: chain_id.parse()?,
            address,
            denom: denom.parse()?,
            amount: amount_bytes.into(),
            operation_type: operation_type.parse()?,
            transaction_hash,
            created_at,
        })
    }
}

impl TryFrom<DbRow> for ChainState {
    type Error = Error;

    fn try_from(row: DbRow) -> Result<Self, Self::Error> {
        let id: String = row.try_get("id")?;
        let node_id: String = row.try_get("node_id")?;
        let config: Json<ChainConfig> = row.try_get("config")?;
        let consensus_timestamp: DateTime<Utc> = row.try_get("consensus_timestamp")?;
        let sequence: u32 = row.try_get("sequence")?;
        let packet_sequence: u32 = row.try_get("packet_sequence")?;
        let connection_details: Option<Json<ConnectionDetails>> =
            row.try_get("connection_details")?;
        let created_at: DateTime<Utc> = row.try_get("created_at")?;
        let updated_at: DateTime<Utc> = row.try_get("updated_at")?;

        Ok(Self {
            id: id.parse()?,
            node_id: node_id.parse()?,
            config: config.0,
            consensus_timestamp,
            sequence,
            packet_sequence,
            connection_details: connection_details.map(|x| x.0),
            created_at,
            updated_at,
        })
    }
}

impl TryFrom<DbRow> for ChainKey {
    type Error = Error;

    fn try_from(row: DbRow) -> Result<Self, Self::Error> {
        let id: i64 = row.try_get("id")?;
        let chain_id: String = row.try_get("chain_id")?;
        let public_key: String = row.try_get("public_key")?;
        let created_at: DateTime<Utc> = row.try_get("created_at")?;

        Ok(Self {
            id,
            chain_id: chain_id.parse()?,
            public_key,
            created_at,
        })
    }
}

impl TryFrom<DbRow> for IbcData {
    type Error = Error;

    fn try_from(row: DbRow) -> Result<Self, Self::Error> {
        let path: String = row.try_get("path")?;
        let data: Vec<u8> = row.try_get("data")?;

        Ok(Self { path, data })
    }
}
