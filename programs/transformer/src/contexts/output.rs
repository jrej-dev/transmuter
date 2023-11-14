use crate::structs::Transmuter;
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct AddOutput<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(
        mut,
        seeds = [b"transmuter", creator.key.as_ref(), seed.to_le_bytes().as_ref()],
        bump,
    )]
    pub transmuter: Box<Account<'info, Transmuter>>,
}

pub struct ClearOutput {}
