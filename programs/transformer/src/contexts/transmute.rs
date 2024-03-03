use crate::errors::TransmuterError;
use crate::structs::Transmuter;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::{invoke, invoke_signed};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, set_authority, Mint, MintTo, SetAuthority, Token, TokenAccount},
};

use mpl_token_metadata::instructions::{
    CreateMasterEditionV3CpiBuilder, CreateMetadataAccountV3CpiBuilder, UpdateV1CpiBuilder,
};
use mpl_token_metadata::types::DataV2;
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
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub sysvar_instructions: AccountInfo<'info>,
}

pub struct TransmuteParams {
    pub seed: u64,
    pub input_json: String,
}

impl<'info> Transmute<'info> {
    pub fn mint_token(&self, mint: &AccountInfo<'info>, ata: &AccountInfo<'info>) -> Result<()> {
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

        mint_to(mint_ctx, 1);
        Ok(())
    }

    pub fn create_metadata(
        &self,
        title: &String,
        symbol: &String,
        uri: &String,
        seller_fee_basis_point: u16,
        metadata: &AccountInfo<'info>,
        mint: &AccountInfo<'info>,
    ) -> Result<()> {
        let seeds = &[&b"auth"[..], &[self.transmuter.auth_bump]];
        let signer_seeds = &[&seeds[..]];

        let data: DataV2 = DataV2 {
            name: title.to_string(),
            symbol: symbol.to_string(),
            uri: uri.to_string(),
            seller_fee_basis_points: seller_fee_basis_point,
            creators: None,
            collection: None,
            uses: None,
        };

        CreateMetadataAccountV3CpiBuilder::new(&self.token_metadata_program)
            .metadata(&metadata.to_account_info())
            .mint(&mint.to_account_info())
            .mint_authority(&self.auth.to_account_info())
            .payer(&self.user.to_account_info())
            .update_authority(&self.auth.to_account_info(), true)
            .system_program(&self.system_program)
            .rent(Some(&self.rent))
            .data(data)
            .is_mutable(true)
            .invoke_signed(signer_seeds);

        Ok(())
    }

    pub fn create_master_edition(
        &self,
        master_edition: &AccountInfo<'info>,
        mint: &AccountInfo<'info>,
        metadata: &AccountInfo<'info>,
    ) -> Result<()> {
        let seeds = &[&b"auth"[..], &[self.transmuter.auth_bump]];
        let signer_seeds = &[&seeds[..]];

        CreateMasterEditionV3CpiBuilder::new(&self.token_metadata_program)
            .edition(&master_edition.to_account_info())
            .mint(&mint.to_account_info())
            .update_authority(&self.auth.to_account_info())
            .mint_authority(&self.auth.to_account_info())
            .payer(&self.user.to_account_info())
            .metadata(&metadata.to_account_info())
            .max_supply(1)
            .token_program(&self.token_program)
            .system_program(&self.system_program)
            .rent(Some(&self.rent))
            .invoke_signed(signer_seeds);

        Ok(())
    }

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
