#![cfg_attr(feature = "doc", feature(doc_cfg))]
#![cfg_attr(coverage, feature(no_coverage))]

#[cfg(not(any(feature = "wasm", feature = "non-wasm",)))]
compile_error!("one of the features ['wasm', 'non-wasm'] must be enabled");

#[cfg(all(feature = "wasm", feature = "non-wasm",))]
compile_error!("only one of the features ['wasm', 'non-wasm'] can be enabled");

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub mod event;
pub mod service;
pub mod signer;
pub mod stag;
pub mod storage;
pub mod tendermint;
pub mod time_util;
pub mod trait_util;
pub mod transaction_builder;
pub mod types;
