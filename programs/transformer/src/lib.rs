use anchor_lang::{prelude::*, solana_program::native_token::LAMPORTS_PER_SOL, system_program};
use anchor_spl::{
    associated_token, token,
    token::{
        burn, set_authority, spl_token::instruction::AuthorityType, transfer, Burn,
        InitializeAccount, SetAuthority, TokenAccount, Transfer,
    },
};

use mpl_token_metadata::accounts::Metadata;
use multimap::MultiMap;

use std::{collections::HashMap, str::FromStr};

use url::Url;

mod contexts;
use contexts::*;

mod errors;
use errors::TransmuterError;

mod structs;
use structs::*;

mod utils;
use utils::*;

use spl_token::solana_program::program::invoke_signed;

declare_id!("H8SJKV7T4egtcwoA2HqSCNYeqrTJuA7SDSeZNrAgMmpf");

#[program]
pub mod transformer {
    use super::*;

    // Creator methods
    pub fn create(
        ctx: Context<Create>,
        seed: u64,
        nft_indexer_json: String,
        input_json: String,
        output_json: String,
        traits_uri: String,
    ) -> Result<()> {
        let nft_indexes = parse_json::<Indexes>(&nft_indexer_json)?;

        if nft_indexes.len() > 0 {
            for index in 0..nft_indexes.len() {
                let current_nft_indexes = &nft_indexes[index];

                let current_ata_index = current_nft_indexes.ata.unwrap();
                let ata = &ctx.remaining_accounts[current_ata_index].to_account_info();
                let mut ata_data: &[u8] = &ata.try_borrow_data()?;
                let deserialized_ata = TokenAccount::try_deserialize(&mut ata_data)?;

                require!(
                    deserialized_ata.owner.key() == ctx.accounts.creator.key()
                        && deserialized_ata.amount == 1,
                    TransmuterError::InvalidNFTOwner
                );

                let metadata: Metadata = Metadata::try_from(
                    &ctx.remaining_accounts[current_nft_indexes.metadata].to_account_info(),
                )?;
                let collection_pubkey = metadata.collection.unwrap().key;
                require!(
                    collection_pubkey.key() == collection_pubkey.key(),
                    TransmuterError::InvalidNFTOwner
                );
            }
        } else {
            //Fee 0.75 SOL
            ctx.accounts
                .pay_fee(ctx.accounts.owner.to_account_info(), 75000000);

            //Fee 0.25 SOL
            ctx.accounts
                .pay_fee(ctx.accounts.wba.to_account_info(), 25000000);
        }

        let transmuter = &mut ctx.accounts.transmuter;
        transmuter.seed = seed;
        transmuter.creator = ctx.accounts.creator.as_ref().key();
        transmuter.auth_bump = ctx.bumps.auth;
        transmuter.transmuter_bump = ctx.bumps.transmuter;

        transmuter.inputs = input_json;
        transmuter.outputs = output_json;
        transmuter.traits_uri = traits_uri;

        Ok(())
    }

    pub fn close(ctx: Context<Close>) -> Result<()> {
        // Prevent any new transmutation
        // Still allow ongoing ones
        msg!("THIS IS A TEST");
        Ok(())
    }

    // User methods
    pub fn send_input<'info>(ctx: Context<SendInput>, seed: u64, vault_seed: u64) -> Result<()> {
        let transmuter = &ctx.accounts.transmuter;
        let transmuter_inputs = parse_json::<InputInfo>(&transmuter.inputs)?;

        ctx.accounts.vault_auth.vault_auth_bump = ctx.bumps.vault_auth;
        let vault_auth = &ctx.accounts.vault_auth;
        // // Init on first input
        // let is_first_input = vault_auth.handled_input_indexes.len() == 0;
        // if is_first_input {
        //     ctx.accounts.vault_auth.user_lock = false;
        //     ctx.accounts.vault_auth.creator_lock = true;
        // }

        //Find an input_info match
        let mut is_match = false;
        for index in 0..transmuter_inputs.len() {
            if vault_auth.handled_input_indexes.contains(&(index as u8)) {
                msg!("Index {:?} already exist in vault_auth", index);
                continue;
            }

            is_match = ctx.accounts.is_matching(&transmuter_inputs[index])?;

            if is_match {
                let is_first_match = vault_auth.handled_input_indexes.len() == 0;
                if is_first_match {
                    ctx.accounts.vault_auth.user_lock = false;
                    ctx.accounts.vault_auth.creator_lock = true;
                }

                ctx.accounts
                    .vault_auth
                    .handled_input_indexes
                    .push(index as u8);

                break;
            }
        }

        require!(is_match, TransmuterError::InvalidInputAccount);

        let cpi_accounts = Transfer {
            from: ctx.accounts.ata.to_account_info(),
            to: ctx.accounts.vault.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        transfer(CpiContext::new(cpi_program, cpi_accounts), 1)?;

        Ok(())
    }

    // pub fn abort(ctx: Context<SendTransmuteInput>) -> Result<()> {
    //     // Send back all nfts from vault auth back to original owner
    //     // Not possible once claim has been initiated
    //     Ok(())
    // }

    pub fn claim_output<'info>(
        ctx: Context<'_, '_, '_, 'info, ClaimOutput<'info>>,
        seed: u64,
        vault_seed: u64
    ) -> Result<()> {
        msg!("TRANSMUTE");
        let transmuter = &ctx.accounts.transmuter;
        let transmuter_inputs = parse_json::<InputInfo>(&transmuter.inputs)?;
        let transmuter_outputs = parse_json::<OutputInfo>(&transmuter.outputs)?;
        let vault_auth = &ctx.accounts.vault_auth;

        //Will need to call that several time
        //     // Reset once claim is over

        let all_inputs_handled = vault_auth.handled_input_indexes.len() == transmuter_inputs.len();
        require!(all_inputs_handled, TransmuterError::InvalidInputLength);

        let all_outputs_handled =
            vault_auth.handled_output_indexes.len() == transmuter_outputs.len();
        require!(!all_outputs_handled, TransmuterError::TransmutationComplete);

        //Find output to use
        for index in 0..transmuter_outputs.len() {
            if vault_auth.handled_output_indexes.contains(&(index as u8)) {
                continue;
            }

            //handle output
            let output_info: &OutputInfo = &transmuter_outputs[index];
            let amount = output_info.amount;

            let mint_account = &ctx.accounts.mint;
            let ata_account = &ctx.accounts.ata;
            let metadata_account = &ctx.accounts.metadata;
            let master_edition_account = &ctx.accounts.master_edition;

            if output_info.rule.is_some() {
                msg!("There is an output rule");
                let rule = output_info.rule.as_ref().unwrap();
                msg!("rule.name: {:?}", rule.name);
                let mint_info = output_info.mint.as_ref().unwrap();

                //         if rule.name == "split" {
                //             msg!("Split rule");

                //             //There should be as much output mints as trait_types
                //             require!(
                //                 rule.trait_types.len() as u64 == output_info.amount,
                //                 TransmuterError::RuleNotApplied,
                //             );

                //             //Find metadata from input
                //             for j in 0..transmuter_inputs.len() {
                //                 let current_input_indexes = &input_indexes[j];

                //                 let input_metadata: Metadata = Metadata::try_from(
                //                     &ctx.remaining_accounts[current_input_indexes.metadata]
                //                         .to_account_info(),
                //                 )?;

                //                 let parsed_url = Url::parse(&input_metadata.uri).unwrap();
                //                 let hash_query: Vec<_> = parsed_url.query_pairs().into_owned().collect();

                //                 for (key, value) in hash_query.clone().iter() {
                //                     if rule
                //                         .trait_types
                //                         .iter()
                //                         .any(|(trait_key, trait_value)| trait_key == key)
                //                     {
                //                         msg!("key: {:?}, val: {:?}", key, value);

                //                         //HERE
                //                         msg!("trait found");
                //                         msg!("use mint uri: {:?}", mint_info.uri);
                //                         let mint_uri = mint_info.uri.to_owned() + "?" + key + "=" + value;
                //                         msg!("new mint uri: {:?}", mint_uri);

                //                         //TODO apply this to mint
                //                         let output_collection = &output_info.collection;

                //                         //mint as much as input traits (max output)
                //                         &ctx.accounts.mint_token(mint_account, ata_account);
                //                         &ctx.accounts.create_metadata(
                //                             &mint_info.title,
                //                             &mint_info.symbol,
                //                             &mint_uri,
                //                             &output_info.collection,
                //                             500,
                //                             metadata_account,
                //                             mint_account,
                //                         );
                //                         &ctx.accounts.create_master_edition(
                //                             master_edition_account,
                //                             mint_account,
                //                             metadata_account,
                //                         );
                //                         &ctx.accounts
                //                             .update_authority(metadata_account, mint_account);

                //                         current_index += 1;
                //                     }
                //                 }
                //             }
                //         } else if rule.name == "merge" {
                //             let mut trait_values: Vec<(String, String)> = Vec::new();
                //             for (key, value) in rule.trait_types.clone().into_iter() {
                //                 for j in 0..transmuter_inputs.len() {
                //                     let current_input_indexes = &input_indexes[j];

                //                     let input_metadata: Metadata = Metadata::try_from(
                //                         &ctx.remaining_accounts[current_input_indexes.metadata]
                //                             .to_account_info(),
                //                     )?;

                //                     let parsed_url = Url::parse(&input_metadata.uri).unwrap();
                //                     let hash_query: Vec<_> =
                //                         parsed_url.query_pairs().into_owned().collect();

                //                     for (query_key, query_value) in hash_query.iter() {
                //                         if query_key == &key {
                //                             trait_values.push((key.clone(), String::from(query_value)));
                //                             break;
                //                         }
                //                     }
                //                 }
                //             }

                //             let uri_traits = trait_values
                //                 .into_iter()
                //                 .map(|trait_value| trait_value.0 + "=" + &trait_value.1 + "&")
                //                 .rev()
                //                 .collect::<Vec<_>>()
                //                 .connect("");

                //             let uri = mint_info.uri.to_owned() + "?" + &uri_traits[0..uri_traits.len() - 1];

                //             &ctx.accounts.mint_token(mint_account, ata_account);
                //             &ctx.accounts.create_metadata(
                //                 &mint_info.title,
                //                 &mint_info.symbol,
                //                 &uri,
                //                 &output_info.collection,
                //                 500,
                //                 metadata_account,
                //                 mint_account,
                //             );
                //             &ctx.accounts.create_master_edition(
                //                 master_edition_account,
                //                 mint_account,
                //                 metadata_account,
                //             );
                //             &ctx.accounts
                //                 .update_authority(metadata_account, mint_account);
                //         } else {
                //             msg!("Rule not found");
                //         }
            } else {
                //         //ADD COLLECTION

                msg!("There is no rule");
                let mint_info = output_info.mint.as_ref().unwrap();
                &ctx.accounts.mint_token();
                &ctx.accounts.create_metadata(
                    &mint_info.title,
                    &mint_info.symbol,
                    &mint_info.uri,
                    &output_info.collection,
                    500,
                );
                &ctx.accounts.create_master_edition();
                &ctx.accounts.update_authority();
            }
        }

        //If 1 mint is successful =>
        //Lock user & unlock creator
        ctx.accounts.vault_auth.user_lock = true;
        ctx.accounts.vault_auth.creator_lock = false;

        Ok(())
    }

    pub fn resolve_input<'info>(ctx: Context<ResolveInput>, seed: u64, vault_seed: u64) -> Result<()> {
        //CREATOR function
        let transmuter = &ctx.accounts.transmuter;
        let transmuter_inputs = parse_json::<InputInfo>(&transmuter.inputs)?;

        let vault_auth = &ctx.accounts.vault_auth;

        require!(
            !vault_auth.creator_lock,
            TransmuterError::TransmutationIncomplete
        );

        //Find an input_info match
        let mut handled_input_index: Option<usize> = None;
        let mut input_info_option: Option<&InputInfo> = None;

        for index in 0..transmuter_inputs.len() {
            if !vault_auth.handled_input_indexes.contains(&(index as u8)) {
                msg!("Index {:?} does not exist in vault_auth", index);
                continue;
            }

            let is_match = ctx.accounts.is_matching(&transmuter_inputs[index])?;

            if is_match {
                input_info_option = Some(&transmuter_inputs[index]);
                handled_input_index = Some(
                    vault_auth
                        .handled_input_indexes
                        .iter()
                        .position(|x| *x == index as u8)
                        .unwrap(),
                );
                break;
            }
        }

        require!(
            !input_info_option.is_none(),
            TransmuterError::InvalidInputAccount
        );
        require!(
            !handled_input_index.is_none(),
            TransmuterError::InvalidInputAccount
        );

        let input_info = input_info_option.unwrap();
        
        let vault_seed_bytes = vault_seed.to_le_bytes();
        let seeds = &[
            b"vaultAuth",
            ctx.accounts.transmuter.to_account_info().key.as_ref(),
            ctx.accounts.user.to_account_info().key.as_ref(),
            &vault_seed_bytes.as_ref(),
            &[vault_auth.vault_auth_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        if input_info.method.as_str() == "transfer" {
            let cpi_accounts = Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.creator_ata.to_account_info(),
                authority: ctx.accounts.vault_auth.to_account_info(),
            };

            let cpi_program = ctx.accounts.token_program.to_account_info();
            transfer(
                CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds),
                1,
            )?;
        } else if input_info.method.as_str() == "burn" {
            let cpi_accounts = Burn {
                mint: ctx.accounts.mint.to_account_info(),
                from: ctx.accounts.vault.to_account_info(),
                authority: ctx.accounts.vault_auth.to_account_info(),
            };

            let cpi_program = ctx.accounts.token_program.to_account_info();
            burn(
                CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds),
                1,
            )?;
        }

        // vault_auth
        //     .handled_input_indexes
        //     .remove(handled_input_index.unwrap());

        Ok(())
    }
}

/*
create transformer with info about the mint
1 input
5 outputs

then when transmute there are the following transactions:
- 1 to verify the input is correct and put it in escrow
- 5 transaction to mint the outputs into the escrow
- if all succeed escrow is resolved of not, input is sent back to original owner, outputs are burned
*/
