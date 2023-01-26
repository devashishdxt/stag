use anyhow::{anyhow, ensure, Result};
use tendermint::abci::{Event as AbciEvent, EventAttribute};
use tendermint_rpc::endpoint::broadcast::tx_commit::Response as TxCommitResponse;

pub fn extract_attribute(events: &[AbciEvent], event_type: &str, key: &str) -> Result<String> {
    let mut attribute = None;

    for event in events {
        if event.kind == event_type {
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

fn get_attribute(tags: &[EventAttribute], key: &str) -> Result<String> {
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

pub fn get_packet_acknowledgement(events: &[AbciEvent]) -> Result<serde_json::Value> {
    let acknowledgement = extract_attribute(events, "write_acknowledgement", "packet_ack")?;
    let acknowledgement: serde_json::Value = serde_json::from_str(&acknowledgement)?;

    let result = acknowledgement.get("result");
    let error = acknowledgement.get("error");

    match (result, error) {
        (None, None) => Err(anyhow!(
            "`result` and `error` are both missing in acknowledgement: {}",
            acknowledgement
        )),
        (Some(result), _) => Ok(result.clone()),
        (None, Some(error)) => Err(anyhow!("acknowledgement contains error: {}", error)),
    }
}
