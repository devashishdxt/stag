use cosmos_sdk_proto::cosmos::staking::v1beta1::MsgUndelegate;

const TYPE_URL: &str = "/cosmos.staking.v1beta1.MsgUndelegate";

impl_any_conversion!(MsgUndelegate, TYPE_URL);
