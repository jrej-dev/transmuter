use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct TransmuterClose<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
}
