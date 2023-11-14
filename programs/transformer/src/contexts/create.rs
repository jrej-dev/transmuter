use crate::structs::Transmuter;
use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::Token};

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Create<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(
        seeds = [b"auth"],
        bump
    )]
    /// CHECK: This is not dangerous because this account doesn't exist
    pub auth: UncheckedAccount<'info>,
    #[account(
        init,
        payer = creator,
        seeds = [b"transmuter", creator.key.as_ref(), seed.to_le_bytes().as_ref()],
        bump,
        space = Transmuter::LEN,
    )]
    pub transmuter: Box<Account<'info, Transmuter>>,
    pub system_program: Program<'info, System>,
}

pub struct CreateWithMintParams {
    pub seed: u64,
}
