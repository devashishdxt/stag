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

#[macro_use]
mod proto;

mod cosmos;
mod ics;
mod public_key;
mod signer;
mod transaction_broadcaster;

pub use self::{
    ics::core::ics24_host::identifier::ChainId,
    public_key::{PublicKey, PublicKeyAlgo},
    signer::{GetPublicKey, Message, Signer},
};
