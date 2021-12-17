use anyhow::Result;
use primitive_types::U256;
use rust_decimal::Decimal;

use crate::{
    event::{EventHandler, NoopEventHandler},
    service::chain_service::ChainService,
    signer::Signer,
    storage::TransactionProvider,
    ChainConfig, ChainId, ChainKey, ChainState, EventHandlerConfig, Identifier, SignerConfig,
    StorageConfig,
};

pub struct Stag<TP, S, EH>
where
    TP: TransactionProvider,
    S: Signer,
    EH: EventHandler,
{
    signer: S,

    chain_service: ChainService<TP, EH>,
}

impl<TP, S> Stag<TP, S, NoopEventHandler>
where
    TP: TransactionProvider,
    S: Signer,
{
    /// Creates a new instance of IBC Solo Machine
    pub async fn new<TC, SC>(storage_config: TC, signer_config: SC) -> Result<Self>
    where
        TC: StorageConfig<Storage = TP>,
        SC: SignerConfig<Signer = S>,
    {
        let storage = storage_config.into_storage().await?;
        let signer = signer_config.into_signer();

        let chain_service = ChainService::new(storage);

        Ok(Self {
            signer,
            chain_service,
        })
    }
}

/// IBC Solo Machine
impl<TP, S, EH> Stag<TP, S, EH>
where
    TP: TransactionProvider + Clone,
    S: Signer,
    EH: EventHandler,
{
    /// Adds an event handler to IBC Solo Machine
    pub fn with_event_handler<EHC>(self, event_handler: EHC) -> Stag<TP, S, EHC::EventHandler>
    where
        EHC: EventHandlerConfig,
    {
        Stag {
            signer: self.signer,
            chain_service: self
                .chain_service
                .with_event_handler(event_handler.into_event_handler()),
        }
    }

    /// Adds metadata of a given chain
    pub async fn add_chain(&self, chain_config: ChainConfig) -> Result<ChainId> {
        self.chain_service.add(&chain_config, &self.signer).await
    }

    /// Gets current stored state for a given chain
    pub async fn get_chain(&self, chain_id: &ChainId) -> Result<Option<ChainState>> {
        self.chain_service.get(chain_id).await
    }

    /// Get on-chain balance of given denom
    pub async fn get_balance(&self, chain_id: &ChainId, denom: &Identifier) -> Result<Decimal> {
        self.chain_service
            .balance(&self.signer, chain_id, denom)
            .await
    }

    /// Gets the final denom of a token on solo machine after sending it on given chain
    pub async fn get_ibc_denom(&self, chain_id: &ChainId, denom: &Identifier) -> Result<String> {
        self.chain_service.get_ibc_denom(chain_id, denom).await
    }

    /// Get all the historical public keys associated with solo machine client on given chain
    ///
    /// TODO: Add `limit` and `offset` parameters to fetch only a subset of keys
    pub async fn get_public_keys(&self, chain_id: &ChainId) -> Result<Vec<ChainKey>> {
        self.chain_service.get_public_keys(chain_id).await
    }

    /// Establishes connection with given chain
    pub async fn connect(
        &self,
        chain_id: ChainId,
        request_id: Option<String>,
        memo: String,
        force: bool,
    ) -> Result<()> {
        todo!()
    }

    /// Mints tokens on given chain
    pub async fn mint(
        &self,
        chain_id: ChainId,
        request_id: Option<String>,
        amount: U256,
        denom: Identifier,
        receiver: Option<String>,
        memo: String,
    ) -> Result<String> {
        todo!()
    }

    /// Burns tokens on given chain
    pub async fn burn(
        &self,
        chain_id: ChainId,
        request_id: Option<String>,
        amount: U256,
        denom: Identifier,
        memo: String,
    ) -> Result<String> {
        todo!()
    }
}
