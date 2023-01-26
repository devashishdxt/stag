#[cfg(not(feature = "solo-machine-v3"))]
use cosmos_sdk_proto::ibc::lightclients::solomachine::v2::Header;

#[cfg(feature = "solo-machine-v3")]
use crate::types::proto::ibc::lightclients::solomachine::v3::Header;

#[cfg(not(feature = "solo-machine-v3"))]
const TYPE_URL: &str = "/ibc.lightclients.solomachine.v2.Header";

#[cfg(feature = "solo-machine-v3")]
const TYPE_URL: &str = "/ibc.lightclients.solomachine.v3.Header";

impl_any_conversion!(Header, TYPE_URL);
