use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct IbcData {
    pub path: String,
    pub data: Vec<u8>,
}
