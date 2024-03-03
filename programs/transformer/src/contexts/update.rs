use crate::structs::Transmuter;
use anchor_lang::prelude::*;

use anchor_spl::token::{set_authority, SetAuthority, Token};
use mpl_token_metadata::instructions::UpdateV1CpiBuilder;
use spl_token::{instruction::AuthorityType, solana_program::program::invoke_signed};

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct UpdateAuthority<'info> {
    #[account(mut)]
    pub creator: SystemAccount<'info>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        seeds = [b"auth"],
        bump = transmuter.auth_bump
    )]
    /// CHECK: This is not dangerous because this account doesn't exist
    pub auth: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"transmuter", creator.key.as_ref(), seed.to_le_bytes().as_ref()],
        bump,
    )]
    pub transmuter: Box<Account<'info, Transmuter>>,
    pub token_program: Program<'info, Token>,
    /// CHECK: Metaplex will check this
    pub token_metadata_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub sysvar_instructions: AccountInfo<'info>,
}

impl<'info> UpdateAuthority<'info> {
    pub fn update_authority(
        &self,
        metadata: &AccountInfo<'info>,
        mint: &AccountInfo<'info>,
    ) -> Result<()> {
        let seeds = &[&b"auth"[..], &[self.transmuter.auth_bump]];
        let signer_seeds = &[&seeds[..]];

        UpdateV1CpiBuilder::new(&self.token_program.to_account_info())
            .authority(&self.auth.to_account_info())
            .mint(&mint.to_account_info())
            .metadata(&metadata.to_account_info())
            .new_update_authority(self.creator.key())
            .payer(&self.user.to_account_info())
            .system_program(&self.system_program.to_account_info())
            .sysvar_instructions(&self.sysvar_instructions.to_account_info())
            .add_remaining_account(&self.token_metadata_program.to_account_info(), false, false)
            .invoke_signed(signer_seeds);

        Ok(())
    }
}
