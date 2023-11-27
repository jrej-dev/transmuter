use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize)]
pub struct InputInfo {
    pub amount: u64,
    pub collection: String,
    pub method: String,
    pub token_standard: String,
}