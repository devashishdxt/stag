use bounce::Atom;
use stag_api::{
    event::TracingEventHandler, signer::MnemonicSigner, storage::IndexedDb,
    tendermint::ReqwestClient,
};

#[derive(PartialEq, Atom, Clone)]
pub struct StagAtom {
    pub signer: MnemonicSigner,
    pub storage: IndexedDb,
    pub rpc_client: ReqwestClient,
    pub event_handler: TracingEventHandler,
}

impl Default for StagAtom {
    fn default() -> Self {
        Self {
            signer: MnemonicSigner::default(),
            storage: IndexedDb::new("stag-ui"),
            rpc_client: ReqwestClient::default(),
            event_handler: TracingEventHandler::default(),
        }
    }
}
