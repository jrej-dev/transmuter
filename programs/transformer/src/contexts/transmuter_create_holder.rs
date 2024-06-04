use crate::structs::{InputInfo, OutputInfo, Transmuter};
use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

#[derive(Accounts)]
#[instruction(seed: u64, input_length: usize, output_length: usize)]
pub struct TransmuterCreateHolder<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(
        init,
        payer = creator,
        seeds = [b"transmuter", creator.key.as_ref(), seed.to_le_bytes().as_ref()],
        bump,
        space =  Transmuter::LEN + input_length * InputInfo::LEN + output_length * OutputInfo::LEN + 220 * 3,
    )]
    pub transmuter: Box<Account<'info, Transmuter>>,
    #[account(
        seeds = [b"auth", transmuter.key().as_ref()],
        bump
    )]
    /// CHECK: This is not dangerous because this account doesn't exist
    pub auth: UncheckedAccount<'info>,
    #[account(mut)]
    pub holder_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    /// CHECK: fix later
    pub holder_metadata: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}
