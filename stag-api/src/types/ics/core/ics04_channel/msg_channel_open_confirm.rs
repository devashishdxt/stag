use cosmos_sdk_proto::ibc::core::channel::v1::MsgChannelOpenConfirm;

const TYPE_URL: &str = "/ibc.core.channel.v1.MsgChannelOpenConfirm";

impl_any_conversion!(MsgChannelOpenConfirm, TYPE_URL);
