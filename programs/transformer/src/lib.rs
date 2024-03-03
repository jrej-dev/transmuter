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

declare_id!("GTyWp6xRHsSC8QXFYTifGResqVRLt9iGjsifSxNswJtA");

#[program]
pub mod transformer {
    use super::*;

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

    pub fn transmute<'info>(
        ctx: Context<'_, '_, '_, 'info, Transmute<'info>>,
        seed: u64,
        input_indexer_json: String,
        output_indexer_json: String,
    ) -> Result<()> {
        msg!("TRANSMUTE");
        let transmuter = &ctx.accounts.transmuter;

        //Handle input verification
        let input_indexes = parse_json::<Indexes>(&input_indexer_json)?;
        let transmuter_inputs = parse_json::<InputInfo>(&transmuter.inputs)?;

        for index in 0..transmuter_inputs.len() {
            let input_info = &transmuter_inputs[index];
            let current_input_indexes = &input_indexes[index];

            match input_info.token_standard.as_str() {
                "nft" => {
                    let input_metadata: Metadata = Metadata::try_from(
                        &ctx.remaining_accounts[current_input_indexes.metadata].to_account_info(),
                    )?;

                    //Verifying collection
                    let collection_pubkey = input_metadata.collection.unwrap().key;
                    require!(
                        collection_pubkey.to_string() == input_info.collection,
                        TransmuterError::InvalidInputAccount
                    );

                    if input_info.rule.is_some() {
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
                            require!(
                                rule.trait_types.clone().into_iter().all(
                                    |(trait_key, trait_value)| hash_query.clone().into_iter().any(
                                        |(key, value)| &trait_key == &key
                                            && (&trait_value == &value
                                                || &trait_value == &String::from("*"))
                                    )
                                ),
                                TransmuterError::RuleNotApplied
                            );
                        }
                    }
                }
                _ => msg!("Token standard not found"),
            };
        }

        //should handle output and mint here
        let mut current_index = 0;
        let output_indexes = parse_json::<Indexes>(&output_indexer_json)?;
        let transmuter_outputs = parse_json::<OutputInfo>(&transmuter.outputs)?;

        for i in 0..transmuter_outputs.len() {
            let output_info = &transmuter_outputs[i];
            let amount = output_info.amount;

            let current_output_indexes = &output_indexes[current_index];

            let mint_account = &ctx.remaining_accounts[current_output_indexes.mint];

            let current_ata_index = current_output_indexes.ata.unwrap();
            let ata_account = &ctx.remaining_accounts[current_ata_index];

            let metadata_account = &ctx.remaining_accounts[current_output_indexes.metadata];

            let current_master_edition_index = current_output_indexes.master_edition.unwrap();
            let master_edition_account = &ctx.remaining_accounts[current_master_edition_index];

            if output_info.rule.is_some() {
                msg!("There is an output rule");
                let rule = output_info.rule.as_ref().unwrap();
                msg!("rule.name: {:?}", rule.name);

                let mint_info = output_info.mint.as_ref().unwrap();

                if rule.name == "split" {
                    msg!("Split rule");

                    //There should be as much output mints as trait_types
                    require!(
                        rule.trait_types.len() as u64 == output_info.amount,
                        TransmuterError::RuleNotApplied,
                    );

                    //Find metadata from input
                    for j in 0..transmuter_inputs.len() {
                        let current_input_indexes = &input_indexes[j];

                        let input_metadata: Metadata = Metadata::try_from(
                            &ctx.remaining_accounts[current_input_indexes.metadata]
                                .to_account_info(),
                        )?;

                        let parsed_url = Url::parse(&input_metadata.uri).unwrap();
                        let hash_query: Vec<_> = parsed_url.query_pairs().into_owned().collect();

                        for (key, value) in hash_query.clone().iter() {
                            if rule
                                .trait_types
                                .iter()
                                .any(|(trait_key, trait_value)| trait_key == key)
                            {
                                msg!("key: {:?}, val: {:?}", key, value);

                                //HERE
                                msg!("trait found");
                                msg!("use mint uri: {:?}", mint_info.uri);
                                let mint_uri = mint_info.uri.to_owned() + "?" + key + "=" + value;
                                msg!("new mint uri: {:?}", mint_uri);

                                //TODO apply this to mint
                                let output_collection = &output_info.collection;

                                //mint as much as input traits (max output)
                                &ctx.accounts.mint_token(mint_account, ata_account);
                                &ctx.accounts.create_metadata(
                                    &mint_info.title,
                                    &mint_info.symbol,
                                    &mint_uri,
                                    &output_info.collection,
                                    500,
                                    metadata_account,
                                    mint_account,
                                );
                                &ctx.accounts.create_master_edition(
                                    master_edition_account,
                                    mint_account,
                                    metadata_account,
                                );
                                &ctx.accounts
                                    .update_authority(metadata_account, mint_account);

                                current_index += 1;
                            }
                        }
                    }
                } else if rule.name == "merge" {
                    let mut trait_values: Vec<(String, String)> = Vec::new();
                    for (key, value) in rule.trait_types.clone().into_iter() {
                        for j in 0..transmuter_inputs.len() {
                            let current_input_indexes = &input_indexes[j];

                            let input_metadata: Metadata = Metadata::try_from(
                                &ctx.remaining_accounts[current_input_indexes.metadata]
                                    .to_account_info(),
                            )?;

                            let parsed_url = Url::parse(&input_metadata.uri).unwrap();
                            let hash_query: Vec<_> =
                                parsed_url.query_pairs().into_owned().collect();

                            for (query_key, query_value) in hash_query.iter() {
                                if query_key == &key {
                                    trait_values.push((key.clone(), String::from(query_value)));
                                    break;
                                }
                            }
                        }
                    }

                    let uri_traits = trait_values
                        .into_iter()
                        .map(|trait_value| trait_value.0 + "=" + &trait_value.1 + "&")
                        .rev()
                        .collect::<Vec<_>>()
                        .connect("");

                    let uri = mint_info.uri.to_owned() + "?" + &uri_traits[0..uri_traits.len() - 1];

                    &ctx.accounts.mint_token(mint_account, ata_account);
                    &ctx.accounts.create_metadata(
                        &mint_info.title,
                        &mint_info.symbol,
                        &uri,
                        &output_info.collection,
                        500,
                        metadata_account,
                        mint_account,
                    );
                    &ctx.accounts.create_master_edition(
                        master_edition_account,
                        mint_account,
                        metadata_account,
                    );
                    &ctx.accounts
                        .update_authority(metadata_account, mint_account);
                } else {
                    msg!("Rule not found");
                }
            } else {
                //ADD COLLECTION

                msg!("There is no rule");
                let mint_info = output_info.mint.as_ref().unwrap();
                &ctx.accounts.mint_token(mint_account, ata_account);
                &ctx.accounts.create_metadata(
                    &mint_info.title,
                    &mint_info.symbol,
                    &mint_info.uri,
                    &output_info.collection,
                    500,
                    metadata_account,
                    mint_account,
                );
                &ctx.accounts.create_master_edition(
                    master_edition_account,
                    mint_account,
                    metadata_account,
                );
                &ctx.accounts
                    .update_authority(metadata_account, mint_account);
            }
            msg!("END OF LOOP");
        }

        //mint is successful =>
        //Handle input disposal
        let mut tokens_to_transfer: Vec<[&AccountInfo<'_>; 2]> = Vec::new();
        let mut tokens_to_burn: Vec<[&AccountInfo<'_>; 2]> = Vec::new();
        for index in 0..transmuter_inputs.len() {
            let input_info = &transmuter_inputs[index];
            let current_input_indexes = &input_indexes[index];
            match input_info.token_standard.as_str() {
                "nft" => {
                    let mint = &ctx.remaining_accounts[current_input_indexes.mint];
                    let current_ata_index = current_input_indexes.ata.unwrap();
                    let ata = &ctx.remaining_accounts[current_ata_index];

                    if input_info.method == "transfer" {
                        let current_creator_ata_index = current_input_indexes.creator_ata.unwrap();
                        let creator_ata = &ctx.remaining_accounts[current_creator_ata_index];

                        tokens_to_transfer.push([ata, creator_ata]);
                    } else if input_info.method == "burn" {
                        tokens_to_burn.push([mint, ata]);
                    }
                }
                _ => msg!("Token standard not found"),
            };
        }

        if tokens_to_transfer.len() > 0 {
            let transfer_atas = tokens_to_transfer
                .iter()
                .map(|&x| x[0].key().to_string())
                .collect::<Vec<_>>();

            //There should be no duplicates
            require!(
                has_unique_elements(&transfer_atas),
                TransmuterError::DuplicateInputAccount
            );

            //There should be the correct number of inputs
            require!(
                &transfer_atas.len() == &transmuter_inputs.len(),
                TransmuterError::InvalidInputAccount
            );

            //transfer nft to owner
            for index in 0..tokens_to_transfer.len() {
                let cpi_accounts = Transfer {
                    from: tokens_to_transfer[index][0].to_account_info(),
                    to: tokens_to_transfer[index][1].to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                };

                let cpi_program = ctx.accounts.token_program.to_account_info();
                transfer(CpiContext::new(cpi_program, cpi_accounts), 1)?;
            }
        }

        if tokens_to_burn.len() > 0 {
            let burn_atas = tokens_to_burn
                .iter()
                .map(|&x| x[0].key().to_string())
                .collect::<Vec<_>>();

            //There should be no duplicates
            require!(
                has_unique_elements(&burn_atas),
                TransmuterError::DuplicateInputAccount
            );

            //There should be the correct number of inputs
            require!(
                &burn_atas.len() == &transmuter_inputs.len(),
                TransmuterError::InvalidInputAccount
            );

            for index in 0..tokens_to_burn.len() {
                let cpi_accounts = Burn {
                    mint: tokens_to_burn[index][0].to_account_info(),
                    from: tokens_to_burn[index][1].to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                };

                let cpi_program = ctx.accounts.token_program.to_account_info();
                burn(CpiContext::new(cpi_program, cpi_accounts), 1)?;
            }
        }

        Ok(())
    }
}
