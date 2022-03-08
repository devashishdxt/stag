use anyhow::{ensure, Context, Result};
#[cfg(target_arch = "wasm32")]
use stag_api::storage::IndexedDb;
#[cfg(not(target_arch = "wasm32"))]
use stag_api::storage::Sqlite;
use stag_api::{
    event::{EventHandlerConfig, TracingEventHandler},
    signer::{MnemonicSigner, SignerConfig},
    stag::{Stag, StagBuilder},
    storage::StorageConfig,
    tendermint::{JsonRpcConfig, ReqwestClient},
    types::chain_state::{ChainConfig, Fee},
};
use url::Url;

pub const CHAIN_ID: &str = "mars-1";
const MNEMONIC: &str = "practice empty client sauce pistol work ticket casual romance appear army fault palace coyote fox super salute slim catch kite wrist three hedgehog sign";

#[cfg(not(target_arch = "wasm32"))]
pub async fn setup() -> Stag<
    StagBuilder<
        <MnemonicSigner as SignerConfig>::Signer,
        <Sqlite as StorageConfig>::Storage,
        <ReqwestClient as JsonRpcConfig>::Client,
        <TracingEventHandler as EventHandlerConfig>::EventHandler,
    >,
> {
    let builder = Stag::builder()
        .with_storage(Sqlite::new("sqlite::memory:"))
        .await
        .unwrap()
        .with_signer(get_mnemonic_signer())
        .unwrap()
        .with_rpc_client(ReqwestClient)
        .with_event_handler(TracingEventHandler);
    builder.build()
}

#[cfg(target_arch = "wasm32")]
pub async fn setup() -> Stag<
    StagBuilder<
        <MnemonicSigner as SignerConfig>::Signer,
        <IndexedDb as StorageConfig>::Storage,
        <ReqwestClient as JsonRpcConfig>::Client,
        <TracingEventHandler as EventHandlerConfig>::EventHandler,
    >,
> {
    let builder = Stag::builder()
        .with_storage(IndexedDb::new("test"))
        .await
        .unwrap()
        .with_signer(get_mnemonic_signer())
        .unwrap()
        .with_rpc_client(ReqwestClient)
        .with_event_handler(TracingEventHandler);
    builder.build()
}

pub fn get_chain_config() -> ChainConfig {
    ChainConfig {
        grpc_addr: Url::parse(get_grpc_addr()).unwrap(),
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
    MnemonicSigner::new()
        .add_chain_config(CHAIN_ID.parse().unwrap(), MNEMONIC, None, None, None)
        .unwrap()
}

fn parse_trusted_hash(hash: &str) -> Result<[u8; 32]> {
    ensure!(!hash.is_empty(), "empty trusted hash");

    let bytes = hex::decode(hash).context("invalid trusted hash hex bytes")?;
    ensure!(bytes.len() == 32, "trusted hash length should be 32");

    let mut trusted_hash = [0; 32];
    trusted_hash.clone_from_slice(&bytes);

    Ok(trusted_hash)
}

#[cfg(target_arch = "wasm32")]
fn get_grpc_addr() -> &'static str {
    "http://localhost:9091"
}

#[cfg(not(target_arch = "wasm32"))]
fn get_grpc_addr() -> &'static str {
    "http://localhost:9090"
}
