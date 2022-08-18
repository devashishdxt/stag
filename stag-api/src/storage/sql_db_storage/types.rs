use anyhow::Error;
use chrono::{DateTime, Utc};
use sqlx::{types::Json, Row};

use crate::types::{
    chain_state::{ChainConfig, ChainKey, ChainState, ConnectionDetails},
    ibc_data::IbcData,
    operation::{Operation, OperationType},
};

use super::DbRow;

impl TryFrom<DbRow> for Operation {
    type Error = Error;

    fn try_from(row: DbRow) -> Result<Self, Self::Error> {
        let id: i64 = row.try_get("id")?;
        let request_id: Option<String> = row.try_get("request_id")?;
        let chain_id: String = row.try_get("chain_id")?;
        let port_id: String = row.try_get("port_id")?;
        let operation_type: Json<OperationType> = row.try_get("operation_type")?;
        let transaction_hash: String = row.try_get("transaction_hash")?;
        let created_at: DateTime<Utc> = row.try_get("created_at")?;

        Ok(Self {
            id,
            request_id,
            chain_id: chain_id.parse()?,
            port_id: port_id.parse()?,
            operation_type: operation_type.0,
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

        cfg_if::cfg_if! {
            if #[cfg(feature = "postgres-storage")] {
                let sequence: u32 = row.try_get::<i64, _>("sequence")?.try_into()?;
            } else if #[cfg(feature = "sqlite-storage")] {
                let sequence: u32 = row.try_get("sequence")?;
            }
        }

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
            connection_details: connection_details.map(|json| json.0),
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
