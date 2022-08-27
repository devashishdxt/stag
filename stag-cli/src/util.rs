use std::path::Path;

use anyhow::Result;
#[cfg(feature = "postgres-storage")]
use stag_api::storage::Postgres;
#[cfg(feature = "sqlite-storage")]
use stag_api::storage::Sqlite;
use stag_api::{
    event::{EventHandlerConfig, TracingEventHandler},
    signer::{MnemonicSigner, SignerConfig},
    stag::{Stag, StagBuilder},
    storage::StorageConfig,
    tendermint::{JsonRpcConfig, ReqwestClient},
};

use crate::signer_config::SignerConfig as SignerConfigParser;

#[cfg(feature = "sqlite-storage")]
pub type CliStag = Stag<
    StagBuilder<
        <MnemonicSigner as SignerConfig>::Signer,
        <Sqlite as StorageConfig>::Storage,
        <ReqwestClient as JsonRpcConfig>::Client,
        <TracingEventHandler as EventHandlerConfig>::EventHandler,
    >,
>;

#[cfg(feature = "postgres-storage")]
pub type CliStag = Stag<
    StagBuilder<
        <MnemonicSigner as SignerConfig>::Signer,
        <Postgres as StorageConfig>::Storage,
        <ReqwestClient as JsonRpcConfig>::Client,
        <TracingEventHandler as EventHandlerConfig>::EventHandler,
    >,
>;

/// Builds stag instance from signer configuration file and db uri
pub async fn stag(signer: impl AsRef<Path>, db_uri: &str) -> Result<CliStag> {
    let signer = SignerConfigParser::create_mnemonic_signer(signer).await?;

    cfg_if::cfg_if! {
        if #[cfg(feature = "sqlite-storage")] {
            let storage = Sqlite::new(db_uri);
        } else if #[cfg(feature = "postgres-storage")] {
            let storage = Postgres::new(db_uri);
        }
    }

    let stag = Stag::builder()
        .with_signer(signer)?
        .with_storage(storage)
        .await?
        .with_rpc_client(ReqwestClient)
        .with_event_handler(TracingEventHandler)
        .build();

    Ok(stag)
}
