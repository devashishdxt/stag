use std::sync::Arc;

use anyhow::{Context, Error};
use stag_api::{
    signer::{MnemonicSigner, Signer, SignerConfig},
    stag::{Stag, StagContext, WithTransaction},
    storage::TransactionProvider,
    tendermint::JsonRpcClient,
    types::ics::core::ics24_host::identifier::ChainId,
};
use tokio::sync::{Mutex, RwLock};
use tonic::{async_trait, Request, Response, Status};

use crate::proto::mnemonic_signer::{
    mnemonic_signer_server, AddChainConfigRequest, AddChainConfigResponse,
    UpdateChainConfigRequest, UpdateChainConfigResponse,
};

pub struct MnemonicSignerService<C>
where
    C: StagContext + WithTransaction + 'static,
    C::Signer: From<<stag_api::signer::MnemonicSigner as SignerConfig>::Signer> + Signer + Clone,
    C::Storage: TransactionProvider,
    C::RpcClient: JsonRpcClient + Clone,
    C::EventHandler: Clone,
{
    signer: Mutex<MnemonicSigner>,
    stag: Arc<RwLock<Stag<C>>>,
}

impl<C> MnemonicSignerService<C>
where
    C: StagContext + WithTransaction + 'static,
    C::Signer: From<<stag_api::signer::MnemonicSigner as SignerConfig>::Signer> + Signer + Clone,
    C::Storage: TransactionProvider,
    C::RpcClient: JsonRpcClient + Clone,
    C::EventHandler: Clone,
{
    pub fn new(signer: MnemonicSigner, stag: Arc<RwLock<Stag<C>>>) -> Self {
        Self {
            signer: Mutex::new(signer),
            stag,
        }
    }
}

#[async_trait]
impl<C> mnemonic_signer_server::MnemonicSigner for MnemonicSignerService<C>
where
    C: StagContext + WithTransaction + 'static,
    C::Signer: From<<stag_api::signer::MnemonicSigner as SignerConfig>::Signer> + Signer + Clone,
    C::Storage: TransactionProvider,
    C::RpcClient: JsonRpcClient + Clone,
    C::EventHandler: Clone,
{
    async fn add_chain_config(
        &self,
        request: Request<AddChainConfigRequest>,
    ) -> Result<Response<AddChainConfigResponse>, Status> {
        let request = request.into_inner();

        let chain_id = request
            .chain_id
            .parse()
            .context("invalid chain id")
            .map_err(|err| Status::invalid_argument(format!("{err:?}")))?;

        let mnemonic = request.mnemonic;

        let hd_path = request.hd_path;

        let account_prefix = request.account_prefix;

        let algo = request
            .algo
            .map(|algo| algo.parse())
            .transpose()
            .context("invalid algo")
            .map_err(|err: Error| Status::invalid_argument(format!("{err:?}")))?;

        let mut signer = self.signer.lock().await;

        signer
            .add_chain_config(
                chain_id,
                &mnemonic,
                hd_path.as_deref(),
                account_prefix.as_deref(),
                algo,
            )
            .context("failed to add chain config")
            .map_err(|err| Status::internal(format!("{err:?}")))?;

        self.stag
            .write()
            .await
            .set_signer(signer.clone())
            .map_err(|err| Status::internal(format!("{err:?}")))?;

        Ok(Response::new(AddChainConfigResponse {}))
    }

    async fn update_chain_config(
        &self,
        request: Request<UpdateChainConfigRequest>,
    ) -> Result<Response<UpdateChainConfigResponse>, Status> {
        let request = request.into_inner();

        let chain_id: ChainId = request
            .chain_id
            .parse()
            .context("invalid chain id")
            .map_err(|err| Status::invalid_argument(format!("{err:?}")))?;

        let request_id = request.request_id;

        let mnemonic = request.mnemonic;

        let hd_path = request.hd_path;

        let account_prefix = request.account_prefix;

        let algo = request
            .algo
            .map(|algo| algo.parse())
            .transpose()
            .context("invalid algo")
            .map_err(|err: Error| Status::invalid_argument(format!("{err:?}")))?;

        let new_public_key = MnemonicSigner::compute_public_key(
            &mnemonic,
            hd_path.as_deref(),
            account_prefix.as_deref(),
            algo,
        )
        .context("unable to compute new public key")
        .map_err(|err| Status::invalid_argument(format!("{err:?}")))?;

        let memo = request.memo.unwrap_or_default();

        let _ = {
            let stag = self.stag.read().await;

            let chain_state = stag
                .get_chain(&chain_id)
                .await
                .map_err(|err| Status::internal(format!("{err:?}")))?;

            match chain_state {
                Some(chain_state) => {
                    if chain_state.is_connected() {
                        stag.update_signer(chain_id.clone(), request_id, new_public_key, memo)
                            .await
                    } else {
                        Ok(())
                    }
                }
                None => Ok(()),
            }
        }
        .map_err(|err| Status::internal(format!("{err:?}")))?;

        let mut signer = self.signer.lock().await;

        signer
            .update_chain_config(
                chain_id,
                &mnemonic,
                hd_path.as_deref(),
                account_prefix.as_deref(),
                algo,
            )
            .context("failed to add chain config")
            .map_err(|err| Status::internal(format!("{err:?}")))?;

        self.stag
            .write()
            .await
            .set_signer(signer.clone())
            .map_err(|err| Status::internal(format!("{err:?}")))?;

        Ok(Response::new(UpdateChainConfigResponse {}))
    }
}
