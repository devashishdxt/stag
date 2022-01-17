use bounce::Atom;
use stag_api::{signer::MnemonicSigner, storage::IndexedDb, tendermint::ReqwestClient};

#[derive(PartialEq, Atom, Clone)]
pub struct StagAtom {
    pub signer: MnemonicSigner,
    pub storage: IndexedDb,
    pub rpc_client: ReqwestClient,
}

impl Default for StagAtom {
    fn default() -> Self {
        Self {
            signer: MnemonicSigner::default(),
            storage: IndexedDb::new("stag-ui"),
            rpc_client: ReqwestClient::default(),
        }
    }
}
