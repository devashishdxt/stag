pub mod core;
pub mod ica;
#[cfg(feature = "mnemonic-signer")]
pub mod mnemonic_signer;
pub mod query;
pub mod transfer;

use std::{net::SocketAddr, sync::Arc};

use anyhow::Result;
use stag_api::{
    event::TracingEventHandler, signer::MnemonicSigner, stag::Stag, storage::Sqlite,
    tendermint::ReqwestClient,
};
use tokio::sync::RwLock;

use self::{
    core::{core_server::CoreServer, CoreService},
    ica::{
        bank::{ica_bank_server::IcaBankServer, IcaBankService},
        staking::{ica_staking_server::IcaStakingServer, IcaStakingService},
    },
    mnemonic_signer::{mnemonic_signer_server::MnemonicSignerServer, MnemonicSignerService},
    transfer::{transfer_server::TransferServer, TransferService},
};

pub struct Server {
    addr: SocketAddr,
    db_uri: String,
}

impl Server {
    pub fn new(addr: SocketAddr, db_uri: String) -> Self {
        Self { addr, db_uri }
    }

    pub async fn run(&self) -> Result<()> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "mnemonic-signer")] {
                let signer = MnemonicSigner::new();
            } else {
                unreachable!("only mnemonic signer is supported");
            }
        }

        cfg_if::cfg_if! {
            if #[cfg(feature = "sqlite-storage")] {
                let storage = Sqlite::new(&self.db_uri);
            } else {
                unreachable!("only sqlite storage is supported");
            }
        }

        let stag = Arc::new(RwLock::new(
            Stag::builder()
                .with_signer(signer.clone())?
                .with_storage(storage)
                .await?
                .with_rpc_client(ReqwestClient)
                .with_event_handler(TracingEventHandler)
                .build(),
        ));

        let mut service = tonic::transport::Server::builder()
            .add_service(CoreServer::new(CoreService::new(stag.clone())))
            .add_service(TransferServer::new(TransferService::new(stag.clone())))
            .add_service(IcaBankServer::new(IcaBankService::new(stag.clone())))
            .add_service(IcaStakingServer::new(IcaStakingService::new(stag.clone())));

        cfg_if::cfg_if! {
            if #[cfg(feature = "mnemonic-signer")] {
                service = service.add_service(MnemonicSignerServer::new(MnemonicSignerService::new(signer, stag)));
            }
        }

        service.serve(self.addr).await?;

        Ok(())
    }
}
