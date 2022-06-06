use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
/// Data type to store IBC data in database
pub struct IbcData {
    /// Path of IBC data
    pub path: String,
    /// Content of IBC data
    pub data: Vec<u8>,
}
