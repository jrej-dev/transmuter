use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Close<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
}
