#[cfg(not(feature = "solo-machine-v3"))]
use cosmos_sdk_proto::ibc::lightclients::solomachine::v2::ConsensusState;

#[cfg(feature = "solo-machine-v3")]
use crate::types::proto::ibc::lightclients::solomachine::v3::ConsensusState;

#[cfg(not(feature = "solo-machine-v3"))]
const TYPE_URL: &str = "/ibc.lightclients.solomachine.v2.ConsensusState";

#[cfg(feature = "solo-machine-v3")]
const TYPE_URL: &str = "/ibc.lightclients.solomachine.v3.ConsensusState";

impl_any_conversion!(ConsensusState, TYPE_URL);
