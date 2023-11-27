use crate::errors::TransmuterError;
use crate::structs::Transmuter;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::{invoke, invoke_signed};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, Mint, MintTo, Token, TokenAccount},
};

// use image::{imageops, DynamicImage};
use mpl_token_metadata::instruction::{create_master_edition_v3, create_metadata_accounts_v3};
use serde_json::Result as Result_serde;

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Transmute<'info> {
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
        bump = transmuter.transmuter_bump,
    )]
    pub transmuter: Box<Account<'info, Transmuter>>,
    pub token_program: Program<'info, Token>,
    /// CHECK: Metaplex will check this
    pub token_metadata_program: UncheckedAccount<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub rent: AccountInfo<'info>,
}

pub struct TransmuteParams {
    pub seed: u64,
    pub input_json: String,
}

impl<'info> Transmute<'info> {
    pub fn mint_token(&self, mint: AccountInfo<'info>, ata: AccountInfo<'info>) -> Result<()> {
        let seeds = &[&b"auth"[..], &[self.transmuter.auth_bump]];
        let signer_seeds = &[&seeds[..]];

        let cpi_accounts = MintTo {
            mint: mint.to_account_info(),
            to: ata.to_account_info(),
            authority: self.auth.to_account_info(),
        };

        let mint_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );

        mint_to(mint_ctx, 1)?;
        Ok(())
    }

    pub fn create_metadata(
        &self,
        title: String,
        symbol: String,
        uri: String,
        seller_fee_basis_point: u16,
        metadata: AccountInfo<'info>,
        mint: AccountInfo<'info>,
    ) -> Result<()> {
        let seeds = &[&b"auth"[..], &[self.transmuter.auth_bump]];
        let signer_seeds = &[&seeds[..]];

        invoke_signed(
            &create_metadata_accounts_v3(
                self.token_metadata_program.key(),
                metadata.key(),
                mint.key(),
                self.auth.key(),
                self.user.key(),
                self.auth.key(),
                title,
                symbol,
                uri,
                None,
                seller_fee_basis_point,
                true,
                true,
                None,
                None,
                None,
            ),
            &[
                metadata.to_account_info(),
                mint.to_account_info(),
                self.auth.to_account_info(),
                self.user.to_account_info(),
                self.token_metadata_program.to_account_info(),
                self.token_program.to_account_info(),
                self.system_program.to_account_info(),
                self.rent.to_account_info(),
            ],
            signer_seeds,
        );
        Ok(())
    }

    pub fn create_master_edition(
        &self,
        master_edition: AccountInfo<'info>,
        mint: AccountInfo<'info>,
        metadata: AccountInfo<'info>,
    ) -> Result<()> {
        let seeds = &[&b"auth"[..], &[self.transmuter.auth_bump]];
        let signer_seeds = &[&seeds[..]];

        invoke_signed(
            &create_master_edition_v3(
                self.token_metadata_program.key(),
                master_edition.key(),
                mint.key(),
                self.auth.key(),
                self.auth.key(),
                metadata.key(),
                self.user.key(),
                Some(0),
            ),
            &[
                master_edition.to_account_info(),
                metadata.to_account_info(),
                mint.to_account_info(),
                self.auth.to_account_info(),
                self.user.to_account_info(),
                self.token_metadata_program.to_account_info(),
                self.token_program.to_account_info(),
                self.system_program.to_account_info(),
                self.rent.to_account_info(),
            ],
            signer_seeds,
        );
        Ok(())
    }
}
