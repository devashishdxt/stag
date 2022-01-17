//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use std::{assert, assert_eq};

use anyhow::{ensure, Context, Result};
use primitive_types::U256;
use stag_api::{
    signer::MnemonicSigner,
    stag::Stag,
    storage::IndexedDb,
    tendermint::ReqwestClient,
    types::chain_state::{ChainConfig, Fee},
};
use url::Url;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

fn get_chain_config() -> ChainConfig {
    ChainConfig {
        grpc_addr: Url::parse("http://localhost:9091").unwrap(),
        rpc_addr: Url::parse("http://localhost:26657").unwrap(),
        fee: Fee {
            amount: "1000".parse().unwrap(),
            denom: "stake".parse().unwrap(),
            gas_limit: 300000,
        },
        trust_level: "1/3".parse().unwrap(),
        trusting_period: humantime::parse_duration("14 days").unwrap(),
        max_clock_drift: humantime::parse_duration("3 sec").unwrap(),
        rpc_timeout: humantime::parse_duration("60 sec").unwrap(),
        diversifier: "stag".to_string(),
        port_id: "transfer".parse().unwrap(),
        trusted_height: 1,
        trusted_hash: parse_trusted_hash(
            "FD417978FD4BF169E3A7468B229901FDAF935C09D8C612EABAAD87FA909B08C5",
        )
        .unwrap(),
        packet_timeout_height_offset: 10,
    }
}

fn get_mnemonic_signer() -> MnemonicSigner {
    MnemonicSigner::new().add_chain_config(
        "mars-1".parse().unwrap(),
        "practice empty client sauce pistol work ticket casual romance appear army fault palace coyote fox super salute slim catch kite wrist three hedgehog sign",
        None,
        None,
        None,
    ).unwrap()
}

#[wasm_bindgen_test]
async fn test_stag_flow() {
    // Build stag (IBC solo machine)
    let builder = Stag::builder()
        .with_storage(IndexedDb::new("test"))
        .await
        .unwrap()
        .with_signer(get_mnemonic_signer())
        .unwrap()
        .with_rpc_client(ReqwestClient);
    let stag = builder.build();

    // Add chain details
    let chain_id = stag.add_chain(&get_chain_config()).await.unwrap();
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

fn parse_trusted_hash(hash: &str) -> Result<[u8; 32]> {
    ensure!(!hash.is_empty(), "empty trusted hash");

    let bytes = hex::decode(hash).context("invalid trusted hash hex bytes")?;
    ensure!(bytes.len() == 32, "trusted hash length should be 32");

    let mut trusted_hash = [0; 32];
    trusted_hash.clone_from_slice(&bytes);

    Ok(trusted_hash)
}
