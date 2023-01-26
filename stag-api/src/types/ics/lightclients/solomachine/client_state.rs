#[cfg(not(feature = "solo-machine-v3"))]
use cosmos_sdk_proto::ibc::lightclients::solomachine::v2::ClientState;

#[cfg(feature = "solo-machine-v3")]
use crate::types::proto::ibc::lightclients::solomachine::v3::ClientState;

#[cfg(not(feature = "solo-machine-v3"))]
const TYPE_URL: &str = "/ibc.lightclients.solomachine.v2.ClientState";

#[cfg(feature = "solo-machine-v3")]
const TYPE_URL: &str = "/ibc.lightclients.solomachine.v3.ClientState";

impl_any_conversion!(ClientState, TYPE_URL);
