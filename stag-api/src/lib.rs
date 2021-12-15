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

mod signer;
mod stag;
mod storage;
mod tendermint;
mod time_util;
mod types;

pub use self::{
    signer::{GetPublicKey, Message, Signer},
    stag::{Stag, StagConfig},
    types::{
        chain_state::{ChainConfig, ChainState, ConnectionDetails, Fee},
        ics::core::ics24_host::identifier::{
            ChainId, ChannelId, ClientId, ConnectionId, Identifier, PortId,
        },
        public_key::{PublicKey, PublicKeyAlgo},
    },
};
