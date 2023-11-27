use crate::structs::Transmuter;
use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::Token};
use std::str::FromStr;

use crate::errors::TransmuterError;

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
    /// CHECK: This is not dangerous because this account is only receiving SOL
    #[account(mut,address=Pubkey::from_str("CHv326keHnnfBMvNFe1TB9dqNraUnUEBDmeCZJVqLhCi").unwrap())]
    pub owner: UncheckedAccount<'info>,
    /// CHECK: This is not dangerous because this account is only receiving SOL
    #[account(mut,address=Pubkey::from_str("3LSY4UTEFt7V7eGsiaAUDzn3iKAJFBPkYseXpdECFknF").unwrap())]
    pub wba: UncheckedAccount<'info>,
}

pub struct CreateWithMintParams {
    pub seed: u64,
}

impl<'info> Create<'info> {
    pub fn pay_fee(&self, to_account: AccountInfo<'info>, lamports: u64) -> Result<()> {
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &self.creator.key(),
            &to_account.key(),
            lamports,
        );

        let tx = anchor_lang::solana_program::program::invoke(
            &ix,
            &[self.creator.to_account_info(), to_account],
        );

        match tx {
            Ok(res) => res,
            Err(e) => panic!("{}", TransmuterError::CreationFeeError),
        };

        Ok(())
    }
}
