mod storage;
mod transaction;
mod types;

pub use self::{storage::IndexedDbStorage, transaction::IndexedDbTransaction};

const CHAIN_STATE_STORE_NAME: &str = "chain_state";
const CHAIN_KEY_STORE_NAME: &str = "chain_key";
const IBC_DATA_STORE_NAME: &str = "ibc_data";
const OPERATIONS_STORE_NAME: &str = "operations";
