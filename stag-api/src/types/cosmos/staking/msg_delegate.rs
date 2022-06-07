use cosmos_sdk_proto::cosmos::staking::v1beta1::MsgDelegate;

const TYPE_URL: &str = "/cosmos.staking.v1beta1.MsgDelegate";

impl_any_conversion!(MsgDelegate, TYPE_URL);
