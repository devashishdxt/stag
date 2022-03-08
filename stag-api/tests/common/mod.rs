use anyhow::{ensure, Result};
#[cfg(target_arch = "wasm32")]
use stag_api::storage::IndexedDb;
#[cfg(not(target_arch = "wasm32"))]
use stag_api::storage::Sqlite;
use stag_api::{
    event::{EventHandlerConfig, TracingEventHandler},
    signer::{MnemonicSigner, SignerConfig},
    stag::{Stag, StagBuilder},
    storage::StorageConfig,
    tendermint::{JsonRpcConfig, ReqwestClient, TendermintClient},
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

pub async fn get_chain_config() -> Result<ChainConfig> {
    Ok(ChainConfig {
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
        trusted_hash: get_trusted_hash().await?,
        packet_timeout_height_offset: 10,
    })
}

fn get_mnemonic_signer() -> MnemonicSigner {
    MnemonicSigner::new()
        .add_chain_config(CHAIN_ID.parse().unwrap(), MNEMONIC, None, None, None)
        .unwrap()
}

#[cfg(target_arch = "wasm32")]
fn get_grpc_addr() -> &'static str {
    "http://localhost:9091"
}

#[cfg(not(target_arch = "wasm32"))]
fn get_grpc_addr() -> &'static str {
    "http://localhost:9090"
}

async fn get_trusted_hash() -> Result<[u8; 32]> {
    let rpc_client = ReqwestClient.into_client();
    let light_block = rpc_client
        .light_block(&"http://localhost:26657/".parse().unwrap(), Some(1))
        .await?;
    let header_hash = light_block.signed_header.header.hash().as_bytes().to_vec();

    let mut trusted_hash = [0; 32];

    ensure!(header_hash.len() == 32);
    trusted_hash.copy_from_slice(&header_hash);

    Ok(trusted_hash)
}
