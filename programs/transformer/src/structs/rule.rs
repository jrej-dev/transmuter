use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Rule {
    pub name: String,
    pub rule_type: String,
    pub trait_types: Vec<String>
}
