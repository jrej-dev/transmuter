use anchor_lang::prelude::*;

use crate::InputInfo;

#[account]
pub struct Transmuter {
    pub seed: u64,
    pub auth_bump: u8,
    pub transmuter_bump: u8,
    pub vault_bump: u8,
    pub creator: Pubkey,
    pub inputs: String,
    pub outputs: String,
    pub traits_uri: String,
}

//Vector max size?
impl Transmuter {
    pub const LEN: usize = 8 + 8 + 4 * 1 + 32;
}
