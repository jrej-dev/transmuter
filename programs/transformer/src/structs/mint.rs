use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize)]
pub struct MintInfo {
    pub title: String,
    pub symbol: String,
    pub uri: String,
}

impl MintInfo {
    pub const LEN: usize = 16 + 8 + 64;
}
