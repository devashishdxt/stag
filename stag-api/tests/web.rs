//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use std::assert;

use stag_api::{stag::Stag, storage::IndexedDb};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn test_stag_add_chain() {
    let builder = Stag::builder().with_storage(IndexedDb::new("test")).await;
    assert!(builder.is_ok());
    let stag = builder.unwrap().build();

    assert!(stag.clear().await.is_ok());
}
