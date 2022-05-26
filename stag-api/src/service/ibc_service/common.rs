use anyhow::{anyhow, ensure, Result};
use tendermint::abci::{
    tag::{Key, Tag},
    Event as AbciEvent,
};
use tendermint_rpc::endpoint::broadcast::tx_commit::Response as TxCommitResponse;

pub fn extract_attribute(events: &[AbciEvent], event_type: &str, key: &str) -> Result<String> {
    let mut attribute = None;

    for event in events {
        if event.type_str == event_type {
            attribute = Some(get_attribute(&event.attributes, key)?);
        }
    }

    attribute.ok_or_else(|| {
        anyhow!(
            "{}:{} not found in tendermint response events: {:?}",
            event_type,
            key,
            events
        )
    })
}

fn get_attribute(tags: &[Tag], key: &str) -> Result<String> {
    let key: Key = key
        .parse()
        .map_err(|e| anyhow!("unable to parse attribute key `{}`: {}", key, e))?;

    for tag in tags {
        if tag.key == key {
            return Ok(tag.value.to_string());
        }
    }

    Err(anyhow!("{} not found in tags: {:?}", key, tags))
}

pub fn ensure_response_success(response: &TxCommitResponse) -> Result<String> {
    ensure!(
        response.check_tx.code.is_ok(),
        "check_tx response contains error code: {}",
        response.check_tx.log
    );

    ensure!(
        response.deliver_tx.code.is_ok(),
        "deliver_tx response contains error code: {}",
        response.deliver_tx.log
    );

    Ok(response.hash.to_string())
}
