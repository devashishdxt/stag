//! This crate implements API to perform basic IBC solo machine operations
//!
//! ## Usage
//!
//! ```rust,ignore
//! let config: StagConfig = get_stag_config();
//! let stag: Stag = Stag::new(config);
//!
//! let chain: Chain = get_chain();
//!
//! let chain_id: ChainId = stag.add_chain(chain);
//! stag.connect(chain_id);
//!
//! stag.mint(chain_id, 100, "GLD");
//! stag.burn(chain_id, 50, "GLD");
//! ```

mod event;
mod service;
mod signer;
mod stag;
mod storage;
mod tendermint;
mod time_util;
mod types;

pub use self::{
    event::EventHandlerConfig,
    signer::SignerConfig,
    stag::Stag,
    storage::StorageConfig,
    types::{
        chain_state::{ChainConfig, ChainKey, ChainState, ConnectionDetails, Fee},
        ics::core::ics24_host::identifier::{
            ChainId, ChannelId, ClientId, ConnectionId, Identifier, PortId,
        },
        public_key::{PublicKey, PublicKeyAlgo},
    },
};

#[cfg(feature = "tracing-event-handler")]
pub use self::event::TracingEventHandler;

#[cfg(feature = "indexed-db-storage")]
pub use self::storage::IndexedDb;

#[cfg(feature = "mnemonic-signer")]
pub use self::signer::MnemonicSigner;
