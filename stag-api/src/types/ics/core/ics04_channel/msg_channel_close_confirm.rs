use cosmos_sdk_proto::ibc::core::channel::v1::MsgChannelCloseConfirm;

const TYPE_URL: &str = "/ibc.core.channel.v1.MsgChannelCloseConfirm";

impl_any_conversion!(MsgChannelCloseConfirm, TYPE_URL);
