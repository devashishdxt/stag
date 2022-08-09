tonic::include_proto!("mnemonic_signer");

use std::sync::Arc;

use anyhow::{Context, Error};
use stag_api::{
    signer::{MnemonicSigner, SignerConfig},
    stag::{Stag, StagContext},
};
use tokio::sync::{Mutex, RwLock};
use tonic::{async_trait, Request, Response, Status};

pub struct MnemonicSignerService<C>
where
    C: StagContext + 'static,
    C::Signer: From<<stag_api::signer::MnemonicSigner as SignerConfig>::Signer>,
{
    signer: Mutex<MnemonicSigner>,
    stag: Arc<RwLock<Stag<C>>>,
}

impl<C> MnemonicSignerService<C>
where
    C: StagContext + 'static,
    C::Signer: From<<stag_api::signer::MnemonicSigner as SignerConfig>::Signer>,
{
    pub fn new(signer: MnemonicSigner, stag: Arc<RwLock<Stag<C>>>) -> Self {
        Self {
            signer: Mutex::new(signer),
            stag,
        }
    }
}

#[async_trait]
impl<C> self::mnemonic_signer_server::MnemonicSigner for MnemonicSignerService<C>
where
    C: StagContext + 'static,
    C::Signer: From<<stag_api::signer::MnemonicSigner as SignerConfig>::Signer>,
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
            .map_err(|err| Status::invalid_argument(err.to_string()))?;

        let mnemonic = request.mnemonic;

        let hd_path = request.hd_path;

        let account_prefix = request.account_prefix;

        let algo = request
            .algo
            .map(|algo| algo.parse())
            .transpose()
            .context("invalid algo")
            .map_err(|err: Error| Status::invalid_argument(err.to_string()))?;

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
            .map_err(|err| Status::internal(err.to_string()))?;

        self.stag
            .write()
            .await
            .set_signer(signer.clone())
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(AddChainConfigResponse {}))
    }
}
