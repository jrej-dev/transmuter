use crate::errors::TransmuterError;
use crate::structs::{InputInfo, OutputInfo, TraitInfo, Transmuter};
use crate::utils::parse_json;
use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::Token};
use std::str::FromStr;

#[derive(Accounts)]
#[instruction(seed: u64, nft_indexer_json: String, input_json: String, output_json: String)]
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
        space = Transmuter::LEN + parse_json::<InputInfo>(&input_json)?.len() * InputInfo::LEN + parse_json::<OutputInfo>(&output_json)?.len() * OutputInfo::LEN + (220 *3),
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
