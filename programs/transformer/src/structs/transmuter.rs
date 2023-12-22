use anchor_lang::prelude::*;

#[account]
pub struct Transmuter {
    pub seed: u64,
    pub auth_bump: u8,
    pub transmuter_bump: u8,
    pub creator: Pubkey,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub traits: Vec<String>,
    pub rules: Vec<String>,
}

//Vector max size?
impl Transmuter {
    pub const LEN: usize = 8 + 2 * 1 + 20 * 32;
}
