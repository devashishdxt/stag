//! Trait utils

/// Base trait for all traits
#[cfg(feature = "wasm")]
pub trait Base {}

/// Base trait for all traits
#[cfg(not(feature = "wasm"))]
pub trait Base: Send + Sync {}

#[cfg(feature = "wasm")]
impl<T> Base for T {}

#[cfg(not(feature = "wasm"))]
impl<T: Send + Sync> Base for T {}
