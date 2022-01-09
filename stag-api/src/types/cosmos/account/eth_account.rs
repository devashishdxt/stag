use cosmos_sdk_proto::cosmos::auth::v1beta1::BaseAccount;

pub use crate::types::proto::ethermint::types::v1::EthAccount;

pub const TYPE_URL: &str = "/ethermint.types.v1.EthAccount";

impl_any_conversion!(EthAccount, TYPE_URL);

impl EthAccount {
    /// Returns base account for ethermint account
    pub fn get_base_account(&self) -> Option<&BaseAccount> {
        self.base_account.as_ref()
    }
}
