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
pub struct SendInput<'info> {
    #[account(mut)]
    pub creator: SystemAccount<'info>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub ata: Account<'info, TokenAccount>,
    #[account(mut)]
    //TODO fix this
    /// CHECK: This is not dangerous because this account doesn't exist
    pub metadata: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"transmuter", creator.key.as_ref(), seed.to_le_bytes().as_ref()],
        bump = transmuter.transmuter_bump,
    )]
    pub transmuter: Box<Account<'info, Transmuter>>,
    #[account(
        init_if_needed,
        payer = user,
        seeds = [b"vaultAuth", transmuter.key().as_ref(), user.key.as_ref(), vault_seed.to_le_bytes().as_ref()],
        bump,
        space = 10000,
    )]
    pub vault_auth: Box<Account<'info, VaultAuth>>,
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> SendInput<'info> {
    pub fn is_matching(&self, input_info: &InputInfo) -> Result<bool> {
        // is input nft matching input_info?
        let mut is_match = false;

        match input_info.token_standard.as_str() {
            "nft" => {
                let input_metadata: Metadata =
                    Metadata::try_from(&self.metadata.to_account_info())?;
                let collection_pubkey = input_metadata.collection.unwrap().key;

                is_match = collection_pubkey.to_string() == input_info.collection;

                if is_match {
                    if input_info.rule.is_some() {
                        is_match = false;

                        msg!("There is an input rule");
                        let rule = input_info.rule.as_ref().unwrap();
                        msg!("rule.name: {:?}", rule.name);

                        if rule.name == "traits" {
                            msg!("Traits rule");

                            msg!("metadata uri, {}", &input_metadata.uri);
                            let parsed_url = Url::parse(&input_metadata.uri).unwrap();
                            msg!("parsed_url works: {:?}", parsed_url);

                            let hash_query: Vec<_> =
                                parsed_url.query_pairs().into_owned().collect();

                            //verify NFT traits
                            is_match = rule.trait_types.clone().into_iter().all(
                                |(trait_key, trait_value)| {
                                    hash_query.clone().into_iter().any(|(key, value)| {
                                        &trait_key == &key
                                            && (&trait_value == &value
                                                || &trait_value == &String::from("*"))
                                    })
                                },
                            );
                        }
                    } else {
                        msg!("No rules found");
                    }
                }
            }
            _ => msg!("Token standard not found"),
        };

        Ok(is_match)
    }
}
