use anchor_lang::prelude::*;

#[account]
pub struct Transmuter {
    pub creator: Pubkey,
    pub seed: u64,
    pub auth_bump: u8,
    pub transmuter_bump: u8,
    pub vault_bump: u8,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub traits_uri: String,
}

//Vector max size?
impl Transmuter {
    pub const LEN: usize = 8 //Discriminator
    + 32 //Pubkey
    + 8 //u64
    + 1 //u8
    + 1 //u8
    + 1 //u8
    + 128 //String
    + 128//String
    + 24; //String
}
