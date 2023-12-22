use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TraitInfo {
    pub trait_type: String,
    pub value: String,
    pub uri: String,
}
