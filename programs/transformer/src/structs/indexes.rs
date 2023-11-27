use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Indexes {
    pub mint: usize,
    pub metadata: usize,
    pub ata: Option<usize>,
    pub creator_ata: Option<usize>,
}
