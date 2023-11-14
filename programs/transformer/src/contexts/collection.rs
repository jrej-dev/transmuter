use crate::structs::Transmuter;
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct AddCollection<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(
        mut,
        seeds = [b"transmuter", creator.key.as_ref(), seed.to_le_bytes().as_ref()],
        bump,
    )]
    pub transmuter: Box<Account<'info, Transmuter>>,
    /// CHECK: This is not dangerous because this account doesn't exist
    pub candy_machine: UncheckedAccount<'info>,
    #[account(
        init,
        payer = creator,
        seeds = [b"candy_machine", candy_machine.key.as_ref()],
        bump,
        space = 8 + 2 * 1 + 10 * 32 + 2 * 32,
    )]
    /// CHECK: This is not dangerous because this account doesn't exist
    pub authority_pda: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}
