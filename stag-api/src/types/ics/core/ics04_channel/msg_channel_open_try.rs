use cosmos_sdk_proto::ibc::core::channel::v1::MsgChannelOpenTry;

const TYPE_URL: &str = "/ibc.core.channel.v1.MsgChannelOpenTry";

impl_any_conversion!(MsgChannelOpenTry, TYPE_URL);
