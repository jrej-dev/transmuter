use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct InputInfo {
    pub amount: u64,
    pub collection: String,
    pub method: String,
    pub token_standard: String,
}

#[derive(Serialize, Deserialize)]
pub struct InputIndex {
    pub mint: usize,
    pub metadata: usize,
    pub ata: usize,
    pub creator_ata: usize,
}
