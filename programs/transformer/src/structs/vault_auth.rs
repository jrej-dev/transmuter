use anchor_lang::prelude::*;

use crate::InputInfo;

#[account]
pub struct VaultAuth {
    pub handled_input_indexes: Vec<u8>,
    pub handled_output_indexes: Vec<u8>,
    pub vault_auth_bump: u8,
    pub user_lock: bool,
    pub creator_lock: bool,
}

//inputs max length: 5;
//outputs max length: 5;
impl VaultAuth {
    pub const LEN: usize = 8 + 5 * 1 + 5 * 1 + 1;
}
