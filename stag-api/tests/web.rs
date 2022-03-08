//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;

mod common;

use std::{assert, assert_eq};

use primitive_types::U256;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn test_stag_flow() {
    // Build stag (IBC solo machine)
    let stag = common::setup().await;

    // Get chain config
    let chain_config = common::get_chain_config().await;
    assert!(chain_config.is_ok());
    let chain_config = chain_config.unwrap();

    // Add chain details
    let chain_id = stag.add_chain(&chain_config).await.unwrap();
    assert_eq!(chain_id.to_string(), "mars-1");

    let chain_state = stag.get_chain(&chain_id).await.unwrap().unwrap();
    assert_eq!(chain_state.id.to_string(), "mars-1");
    assert!(chain_state.connection_details.is_none());

    // Establish IBC connection
    stag.connect(chain_id.clone(), None, "stag".to_string(), false)
        .await
        .unwrap();

    let chain_state = stag.get_chain(&chain_id).await.unwrap().unwrap();
    assert_eq!(chain_state.id.to_string(), "mars-1");
    assert!(chain_state.connection_details.is_some());

    // Check balance
    let gld_balance = stag
        .get_balance(&chain_id, &"gld".parse().unwrap())
        .await
        .unwrap();
    assert!(gld_balance.is_zero());

    // Mint some tokens
    assert!(stag
        .mint(
            chain_id.clone(),
            None,
            U256::from_dec_str("100").unwrap(),
            "gld".parse().unwrap(),
            None,
            "stag".to_string(),
        )
        .await
        .is_ok());

    // Check balance
    let gld_balance = stag
        .get_balance(&chain_id, &"gld".parse().unwrap())
        .await
        .unwrap();
    assert_eq!(gld_balance, "100".parse().unwrap());

    // Burn some tokens
    assert!(stag
        .burn(
            chain_id.clone(),
            None,
            U256::from_dec_str("50").unwrap(),
            "gld".parse().unwrap(),
            "stag".to_string(),
        )
        .await
        .is_ok());

    // Check balance
    let gld_balance = stag
        .get_balance(&chain_id, &"gld".parse().unwrap())
        .await
        .unwrap();
    assert_eq!(gld_balance, "50".parse().unwrap());

    // Clear database
    assert!(stag.clear().await.is_ok());
}
