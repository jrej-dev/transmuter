use super::mint::MintInfo;
use super::rule::Rule;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct OutputInfo {
    pub amount: u64,
    pub collection: String,
    pub method: String,
    pub token_standard: String,
    pub rule: Option<Rule>,
    pub uri: Option<String>,
    pub mint: Option<MintInfo>,
}

impl OutputInfo {
    pub const LEN: usize = 8 + 44 + 4 + 4 + Rule::LEN + 64 + MintInfo::LEN;
}
