//! Trait utils

/// Base trait for all traits
#[cfg(feature = "wasm")]
pub trait Base {}

/// Base trait for all traits
#[cfg(all(not(feature = "wasm"), feature = "non-wasm"))]
pub trait Base: Send + Sync {}

#[cfg(feature = "wasm")]
impl<T> Base for T {}

#[cfg(all(not(feature = "wasm"), feature = "non-wasm"))]
impl<T: Send + Sync> Base for T {}
