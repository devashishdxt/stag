//! Transaction signers used by Stag API
mod builder;
#[cfg(feature = "mnemonic-signer")]
mod mnemonic_signer;
mod signer_traits;

pub use self::{
    builder::SignerConfig,
    signer_traits::{GetPublicKey, Message, Signer},
};

#[cfg(feature = "mnemonic-signer")]
pub use self::builder::MnemonicSigner;

/// A no-op signer
pub struct NoopSigner;
