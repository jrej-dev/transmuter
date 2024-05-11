use crate::errors::TransmuterError;
use crate::structs::{InputInfo, OutputInfo, TraitInfo, Transmuter};
use crate::utils::parse_json;
use crate::VaultAuth;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};
use mpl_token_metadata::accounts::Metadata;

use std::str::FromStr;
use url::Url;

#[derive(Accounts)]
#[instruction(seed: u64, vault_seed: u64)]
pub struct ResolveInput<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(mut)]
    pub user: SystemAccount<'info>,
    #[account(mut)]
    pub mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub creator_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    /// CHECK: fix later
    pub metadata: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"transmuter", creator.key.as_ref(), seed.to_le_bytes().as_ref()],
        bump = transmuter.transmuter_bump,
    )]
    pub transmuter: Box<Account<'info, Transmuter>>,
    #[account(
        mut,
        seeds = [b"vaultAuth", transmuter.key().as_ref(), user.key.as_ref(), vault_seed.to_le_bytes().as_ref()],
        bump = vault_auth.vault_auth_bump,
    )]
    pub vault_auth: Box<Account<'info, VaultAuth>>,
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}