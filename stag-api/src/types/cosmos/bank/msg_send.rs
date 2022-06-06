use cosmos_sdk_proto::cosmos::bank::v1beta1::MsgSend;

const TYPE_URL: &str = "/cosmos.bank.v1beta1.MsgSend";

impl_any_conversion!(MsgSend, TYPE_URL);
