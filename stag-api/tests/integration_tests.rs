//! Integration test suite

#![cfg(not(target_arch = "wasm32"))]

use primitive_types::U256;
use stag_api::{signer::MnemonicSigner, types::operation::OperationType};

mod common;

#[tokio::test]
async fn test_stag_flow() {
    // Build stag (IBC solo machine)
    let stag = common::setup(common::MNEMONIC_1).await;

    // Get chain config
    let chain_config = common::get_chain_config().await;
    assert!(chain_config.is_ok());
    let chain_config = chain_config.unwrap();

    // Add chain details
    let chain_id = stag.add_chain(&chain_config).await.unwrap();
    assert_eq!(chain_id.to_string(), common::CHAIN_ID);

    let chain_state = stag.get_chain(&chain_id).await.unwrap().unwrap();
    assert_eq!(chain_state.id.to_string(), common::CHAIN_ID);
    assert!(chain_state.connection_details.is_none());

    // Get ibc denom should return error before connection
    assert!(stag
        .get_ibc_denom(&chain_id, &"gld".parse().unwrap())
        .await
        .is_err());

    // Establish IBC connection
    stag.connect(chain_id.clone(), None, "stag".to_string(), false)
        .await
        .unwrap();

    let chain_state = stag.get_chain(&chain_id).await.unwrap().unwrap();
    assert_eq!(chain_state.id.to_string(), common::CHAIN_ID);
    assert!(chain_state.connection_details.is_some());

    // Get ibc denom should return success after connection
    let ibc_denom = stag.get_ibc_denom(&chain_id, &"gld".parse().unwrap()).await;
    assert!(ibc_denom.is_ok());
    let ibc_denom = ibc_denom.unwrap();

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

    // Check history
    let history = stag.get_history(&chain_id, None, None).await;
    assert!(history.is_ok());
    let history = history.unwrap();

    assert_eq!(history.len(), 2);

    // History should be in reverse order
    assert_eq!(history[0].amount, 50u8.into());
    assert_eq!(history[0].denom.to_string(), "gld");
    assert_eq!(history[0].operation_type, OperationType::Burn);

    assert_eq!(history[1].amount, 100u8.into());
    assert_eq!(history[1].denom.to_string(), "gld");
    assert_eq!(history[1].operation_type, OperationType::Mint);

    // Update signer to use new mnemonic
    let new_public_key = common::get_public_key(&chain_id, common::MNEMONIC_2).await;

    assert!(stag
        .update_signer(chain_id.clone(), None, new_public_key, "stag".to_string())
        .await
        .is_ok());

    // Mint should fail with old signer after update
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
        .is_err());

    // Update signer in stag
    let mut stag = stag;
    assert!(stag
        .set_signer(
            MnemonicSigner::new()
                .add_chain_config(chain_id.clone(), common::MNEMONIC_2, None, None, None)
                .unwrap(),
        )
        .is_ok());

    // New ibc denom should be same as old
    let new_ibc_denom = stag.get_ibc_denom(&chain_id, &"gld".parse().unwrap()).await;
    assert!(new_ibc_denom.is_ok());
    assert_eq!(new_ibc_denom.unwrap(), ibc_denom);

    // Mint should succeed with new signer
    stag.mint(
        chain_id.clone(),
        None,
        U256::from_dec_str("100").unwrap(),
        "gld".parse().unwrap(),
        None,
        "stag".to_string(),
    )
    .await
    .unwrap();

    // Get public keys should return two
    let public_keys = stag.get_public_keys(&chain_id, None, None).await;
    assert!(public_keys.is_ok());
    let public_keys = public_keys.unwrap();

    // Should return public keys in reverse order
    assert_eq!(public_keys.len(), 2);
    assert_eq!(public_keys[0].chain_id, chain_id);
    assert_eq!(
        public_keys[0].public_key,
        "03709F114FB5F89416D602474D22B70C3DE08E56A8DBB850161AD91475ABBC84CC"
    );
    assert_eq!(public_keys[1].chain_id, chain_id);
    assert_eq!(
        public_keys[1].public_key,
        "02A94B5772665ECD0A38BC26ECE57A3D15674A12597E223604345C49FB2EFDFD72"
    );

    // Clear database
    assert!(stag.clear().await.is_ok());
}

#[tokio::test]
async fn test_get_all_chains() {
    // Build stag (IBC solo machine)
    let stag = common::setup(common::MNEMONIC_1).await;

    // Get chain config
    let chain_config = common::get_chain_config().await;
    assert!(chain_config.is_ok());
    let chain_config = chain_config.unwrap();

    // No chains yet
    let chains = stag.get_all_chains(None, None).await;
    assert!(chains.is_ok());
    assert!(chains.unwrap().is_empty());

    // Add chain details
    let chain_id = stag.add_chain(&chain_config).await.unwrap();
    assert_eq!(chain_id.to_string(), common::CHAIN_ID);

    // Get all chains should return one
    let chains = stag.get_all_chains(None, None).await;
    assert!(chains.is_ok());
    let chains = chains.unwrap();

    assert_eq!(chains.len(), 1);
    assert_eq!(chains[0].id.to_string(), common::CHAIN_ID);

    // Clear database
    assert!(stag.clear().await.is_ok());
}

#[tokio::test]
async fn test_get_public_keys() {
    // Build stag (IBC solo machine)
    let stag = common::setup(common::MNEMONIC_1).await;

    // Get chain config
    let chain_config = common::get_chain_config().await;
    assert!(chain_config.is_ok());
    let chain_config = chain_config.unwrap();

    // Add chain details
    let chain_id = stag.add_chain(&chain_config).await.unwrap();
    assert_eq!(chain_id.to_string(), common::CHAIN_ID);

    // Get public keys should return one
    let public_keys = stag.get_public_keys(&chain_id, None, None).await;
    assert!(public_keys.is_ok());
    let public_keys = public_keys.unwrap();

    assert_eq!(public_keys.len(), 1);
    assert_eq!(public_keys[0].chain_id, chain_id);
    assert_eq!(
        public_keys[0].public_key,
        "02A94B5772665ECD0A38BC26ECE57A3D15674A12597E223604345C49FB2EFDFD72"
    );

    // Clear database
    assert!(stag.clear().await.is_ok());
}

#[tokio::test]
async fn test_force_reconnection() {
    // Build stag (IBC solo machine)
    let stag = common::setup(common::MNEMONIC_1).await;

    // Get chain config
    let chain_config = common::get_chain_config().await;
    assert!(chain_config.is_ok());
    let chain_config = chain_config.unwrap();

    // Add chain details
    let chain_id = stag.add_chain(&chain_config).await.unwrap();
    assert_eq!(chain_id.to_string(), common::CHAIN_ID);

    let chain_state = stag.get_chain(&chain_id).await.unwrap().unwrap();
    assert_eq!(chain_state.id.to_string(), common::CHAIN_ID);
    assert!(chain_state.connection_details.is_none());

    // Establish IBC connection
    stag.connect(chain_id.clone(), None, "stag".to_string(), false)
        .await
        .unwrap();

    let chain_state = stag.get_chain(&chain_id).await.unwrap().unwrap();
    assert_eq!(chain_state.id.to_string(), common::CHAIN_ID);
    assert!(chain_state.connection_details.is_some());

    // Get ibc denom should return success after connection
    let ibc_denom = stag.get_ibc_denom(&chain_id, &"gld".parse().unwrap()).await;
    assert!(ibc_denom.is_ok());
    let ibc_denom = ibc_denom.unwrap();

    // Establish new IBC connection
    stag.connect(chain_id.clone(), None, "stag".to_string(), true)
        .await
        .unwrap();

    let chain_state = stag.get_chain(&chain_id).await.unwrap().unwrap();
    assert_eq!(chain_state.id.to_string(), common::CHAIN_ID);
    assert!(chain_state.connection_details.is_some());

    // Get ibc denom should return different denom after reconnection
    let new_ibc_denom = stag.get_ibc_denom(&chain_id, &"gld".parse().unwrap()).await;
    assert!(new_ibc_denom.is_ok());
    let new_ibc_denom = new_ibc_denom.unwrap();

    assert!(new_ibc_denom != ibc_denom);
}
