use std::net::SocketAddr;

use anyhow::Result;
use clap::Subcommand;
use stag_api::{
    event::TracingEventHandler, signer::MnemonicSigner, stag::Stag, storage::Sqlite,
    tendermint::ReqwestClient,
};
use tonic::transport::Server;

use crate::service::core::{core_server::CoreServer, CoreService};

#[derive(Subcommand)]
pub enum SubCommand {
    /// Starts Stag gRPC server
    Start {
        /// gRPC server address
        #[clap(short, long, default_value = "0.0.0.0:9000")]
        addr: SocketAddr,
        /// Database connection URI
        #[cfg(feature = "sqlite-storage")]
        #[cfg_attr(
            feature = "sqlite-storage",
            clap(short, long, default_value = "sqlite://solo-machine.db")
        )]
        db_uri: String,
    },
}

impl SubCommand {
    pub async fn run(self) -> Result<()> {
        match self {
            Self::Start { addr, db_uri } => {
                tracing_subscriber::fmt::init();

                #[cfg(feature = "sqlite-storage")]
                let storage = Sqlite::new(&db_uri)?;

                let stag = Stag::builder()
                    .with_signer(MnemonicSigner::new())?
                    .with_storage(storage)
                    .await?
                    .with_rpc_client(ReqwestClient)
                    .with_event_handler(TracingEventHandler)
                    .build();

                Server::builder()
                    .add_service(CoreServer::new(CoreService::new(stag)))
                    .serve(addr)
                    .await?;

                Ok(())
            }
        }
    }
}
