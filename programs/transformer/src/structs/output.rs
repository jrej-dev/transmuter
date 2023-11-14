use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct OutputInfo {
    pub method: String,
    pub token_standard: String,
}