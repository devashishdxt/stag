use std::time::Duration;

use anyhow::{Error, Result};
use stag_api::tendermint::{JsonRpcConfig, ReqwestClient, TendermintClient};
use stag_grpc::{
    proto::{
        core::{
            core_client::CoreClient, AddChainRequest, ConnectChainRequest, CreateChannelRequest,
        },
        mnemonic_signer::{mnemonic_signer_client::MnemonicSignerClient, AddChainConfigRequest},
        query::{
            op::OpType, query_client::QueryClient, BurnOperation, GetBalanceRequest,
            GetHistoryRequest, MintOperation,
        },
        transfer::{transfer_client::TransferClient, BurnRequest, MintRequest},
    },
    Server,
};
use tokio::task::JoinHandle;

const SERVER_ADDR: &str = "[::1]:8000";
const SERVER_URL: &str = "http://[::1]:8000";

pub const CHAIN_ID: &str = "mars-1";
pub const MNEMONIC_1: &str = "practice empty client sauce pistol work ticket casual romance appear army fault palace coyote fox super salute slim catch kite wrist three hedgehog sign";
pub const MNEMONIC_2: &str = "setup chicken slogan define emerge original sugar bitter suggest bicycle increase eager rather end predict relief moment burden lonely ginger umbrella secret toy trash";

struct TestServer {
    handle: JoinHandle<Result<(), Error>>,
}

impl TestServer {
    pub async fn spawn() -> Self {
        let server = Server::new(SERVER_ADDR.parse().unwrap(), "sqlite::memory:".to_string());
        let handle = tokio::spawn(async move { server.run().await });

        tokio::time::sleep(Duration::from_secs(1)).await; // Let the server start

        Self { handle }
    }

    pub async fn stop(self) {
        self.handle.abort();
    }
}

#[tokio::test]
async fn test_stag_grpc_transfer_flow() {
    let test_server = TestServer::spawn().await;

    // Add signer config
    let mut signer_client = MnemonicSignerClient::connect(SERVER_URL)
        .await
        .expect("failed to connect to server");

    signer_client
        .add_chain_config(AddChainConfigRequest {
            chain_id: CHAIN_ID.to_string(),
            mnemonic: MNEMONIC_1.to_string(),
            hd_path: None,
            account_prefix: None,
            algo: None,
        })
        .await
        .expect("failed to add signer config for chain");

    // Add chain config
    let mut core_client = CoreClient::connect(SERVER_URL)
        .await
        .expect("failed to connect to server");

    let chain_id = core_client
        .add_chain(AddChainRequest {
            grpc_addr: None,
            rpc_addr: None,
            fee_config: None,
            trust_level: None,
            trusting_period: None,
            max_clock_drift: None,
            rpc_timeout: None,
            diversifier: None,
            trusted_height: 1,
            trusted_hash: get_trusted_hash()
                .await
                .expect("unable to fetch trusted hash"),
            packet_timeout_height_offset: None,
        })
        .await
        .expect("failed to add chain")
        .into_inner()
        .chain_id;

    assert_eq!(chain_id, CHAIN_ID);

    // Establish IBC connection with chain
    core_client
        .connect_chain(ConnectChainRequest {
            chain_id: CHAIN_ID.to_string(),
            request_id: None,
            memo: None,
            force: false,
        })
        .await
        .expect("failed to establish IBC connection with chain");

    // Create transfer channel
    core_client
        .create_transfer_channel(CreateChannelRequest {
            chain_id: CHAIN_ID.to_string(),
            request_id: None,
            memo: None,
        })
        .await
        .expect("failed to create transfer channel with chain");

    // Check balance of IBC denom
    let mut query_client = QueryClient::connect(SERVER_URL)
        .await
        .expect("failed to connect to server");

    let balance = query_client
        .get_balance(GetBalanceRequest {
            chain_id: CHAIN_ID.to_string(),
            denom: "gld".to_string(),
            ibc_denom: true,
        })
        .await
        .expect("failed to fetch balance")
        .into_inner()
        .balance;

    assert_eq!(balance, "0");

    // Mint some tokens
    let mut transfer_client = TransferClient::connect(SERVER_URL)
        .await
        .expect("failed to connect to server");

    transfer_client
        .mint(MintRequest {
            chain_id: CHAIN_ID.to_string(),
            request_id: None,
            amount: "100".to_string(),
            denom: "gld".to_string(),
            receiver_address: None,
            memo: None,
        })
        .await
        .expect("failed to mint tokens");

    // Check balance after minting
    let balance = query_client
        .get_balance(GetBalanceRequest {
            chain_id: CHAIN_ID.to_string(),
            denom: "gld".to_string(),
            ibc_denom: true,
        })
        .await
        .expect("failed to fetch balance")
        .into_inner()
        .balance;

    assert_eq!(balance, "100");

    // Burn some tokens
    transfer_client
        .burn(BurnRequest {
            chain_id: CHAIN_ID.to_string(),
            request_id: None,
            amount: "50".to_string(),
            denom: "gld".to_string(),
            memo: None,
        })
        .await
        .expect("failed to burn tokens");

    // Check balance after burning
    let balance = query_client
        .get_balance(GetBalanceRequest {
            chain_id: CHAIN_ID.to_string(),
            denom: "gld".to_string(),
            ibc_denom: true,
        })
        .await
        .expect("failed to fetch balance")
        .into_inner()
        .balance;

    assert_eq!(balance, "50");

    // Check history
    let operations = query_client
        .get_history(GetHistoryRequest {
            chain_id: CHAIN_ID.to_string(),
            limit: None,
            offset: None,
        })
        .await
        .expect("failed to fetch history")
        .into_inner()
        .operations;

    assert_eq!(operations.len(), 2);

    // History should be in reverse order
    assert_eq!(
        operations[0].op_type.as_ref().unwrap(),
        &OpType::Burn(BurnOperation {
            from: "cosmos1j2qpprh2xke7qjqzehfqgjdkfgddf9dm06dugw".to_string(),
            denom: "gld".to_string(),
            amount: "50".to_string(),
        })
    );
    assert_eq!(
        operations[1].op_type.as_ref().unwrap(),
        &OpType::Mint(MintOperation {
            to: "cosmos1j2qpprh2xke7qjqzehfqgjdkfgddf9dm06dugw".to_string(),
            denom: "gld".to_string(),
            amount: "100".to_string(),
        })
    );

    test_server.stop().await;
}

async fn get_trusted_hash() -> Result<String> {
    let rpc_client = ReqwestClient.into_client();
    let light_block = rpc_client
        .light_block(&"http://127.0.0.1:26657/".parse().unwrap(), Some(1))
        .await?;
    let trusted_hash = light_block.signed_header.header.hash().as_bytes().to_vec();

    Ok(hex::encode(trusted_hash))
}
