use anchor_lang::prelude::*;
use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;
use anchor_spl::token::{burn, transfer, Burn, InitializeAccount, TokenAccount, Transfer};
use mpl_token_metadata::state::{Metadata, TokenMetadataAccount};
use multimap::MultiMap;
use std::str::FromStr;
use url::Url;

mod contexts;
use contexts::*;

mod errors;
use errors::TransmuterError;

mod structs;
use structs::*;

mod utils;
use utils::*;

declare_id!("GTyWp6xRHsSC8QXFYTifGResqVRLt9iGjsifSxNswJtA");

#[program]
pub mod transformer {
    use super::*;

    pub fn create(ctx: Context<Create>, seed: u64, nft_indexer_json: String) -> Result<()> {
        let nft_indexes = parse_indexes(&nft_indexer_json)?;

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

                let metadata: Metadata = Metadata::from_account_info(
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
        Ok(())
    }

    pub fn add_input(ctx: Context<AddInput>, seed: u64, input_json: String) -> Result<()> {
        let transmuter = &mut ctx.accounts.transmuter;
        transmuter.inputs.push(input_json);
        Ok(())
    }

    pub fn add_output(ctx: Context<AddOutput>, seed: u64, output_json: String) -> Result<()> {
        let transmuter = &mut ctx.accounts.transmuter;
        transmuter.outputs.push(output_json);

        Ok(())
    }

    pub fn add_rule(ctx: Context<AddTraits>, seed: u64, rule_json: String) -> Result<()> {
        let transmuter = &mut ctx.accounts.transmuter;
        transmuter.rules.push(rule_json);
        Ok(())
    }

    pub fn add_trait(ctx: Context<AddTraits>, seed: u64, traits_json: String) -> Result<()> {
        let transmuter = &mut ctx.accounts.transmuter;
        transmuter.traits.push(traits_json);
        Ok(())
    }

    pub fn transmute<'info>(
        ctx: Context<'_, '_, '_, 'info, Transmute<'info>>,
        seed: u64,
        input_indexer_json: String,
        output_indexer_json: String,
    ) -> Result<()> {
        msg!("TRANSMUTE");
        let mut tokens_to_transfer: Vec<[&AccountInfo<'_>; 2]> = Vec::new();
        let mut tokens_to_burn: Vec<[&AccountInfo<'_>; 2]> = Vec::new();
        let transmuter = &ctx.accounts.transmuter;

        //Find rules
        let rules: Vec<Rule> = transmuter
            .rules
            .iter()
            .map(|rule| parse_rule(rule).unwrap())
            .rev()
            .collect();
        let mint_rule_index: Option<usize> = rules.iter().position(|rule| rule.rule_type == "mint");

        //Handle input
        let input_indexes = parse_indexes(&input_indexer_json)?;
        for index in 0..transmuter.inputs.len() {
            let input_info = parse_input(&transmuter.inputs[index])?;
            msg!("token_standard, {}", input_info.token_standard);
            msg!("method, {}", input_info.method);
            msg!("collection, {}", input_info.collection);

            let current_input_indexes = &input_indexes[index];

            match input_info.token_standard.as_str() {
                "nft" => {
                    let mint: &AccountInfo<'_> =
                        &ctx.remaining_accounts[current_input_indexes.mint];
                    let metadata: Metadata = Metadata::from_account_info(
                        &ctx.remaining_accounts[current_input_indexes.metadata].to_account_info(),
                    )?;

                    //Verifying collection
                    let collection_pubkey = metadata.collection.unwrap().key;
                    require!(
                        collection_pubkey.to_string() == input_info.collection,
                        TransmuterError::InvalidInputAccount
                    );

                    //Handle input-based rules
                    if mint_rule_index.is_some() {
                        if rules[mint_rule_index.unwrap()].name == "split" {
                            let trait_types = &rules[mint_rule_index.unwrap()].trait_types;
                            //There should be as much output mints as trait_types otherwise slice

                            msg!("metadata uri, {}", metadata.data.uri);

                            let parsed_url = Url::parse(&metadata.data.uri).unwrap();
                            let hash_query: MultiMap<String, _> =
                                parsed_url.query_pairs().into_owned().collect();

                            let mut filtered_hash_query: MultiMap<String, String> = MultiMap::new();

                            for (key, value) in hash_query.iter() {
                                if trait_types.contains(key) {
                                    filtered_hash_query.insert(key.to_string(), value.to_string());
                                }
                            }

                            require!(
                                filtered_hash_query.len() > 0,
                                TransmuterError::RuleNotApplied,
                            );

                            for (key, value) in filtered_hash_query.iter() {
                                msg!("key: {:?}, val: {:?}", key, value);
                                let found_trait_json = transmuter
                                    .traits
                                    .iter()
                                    .find(|json| &parse_trait(json).unwrap().trait_type == key);

                                let mut found_trait;
                                if found_trait_json.is_some() {
                                    found_trait = parse_trait(found_trait_json.unwrap())?;
                                    msg!("trait uri: {:?}", found_trait.uri)
                                }

                                require!(found_trait.is_some(), TransmuterError::MissingTrait);

                                //create minting json
                                //get collection to add
                                //mint as much as input traits (max output)
                            }
                        }
                    }

                    //Handle input disposal
                    let current_ata_index = current_input_indexes.ata.unwrap();
                    let ata: &AccountInfo<'_> = &ctx.remaining_accounts[current_ata_index];

                    if input_info.method == "transfer" {
                        let current_creator_ata_index = current_input_indexes.creator_ata.unwrap();
                        let creator_ata: &AccountInfo<'_> =
                            &ctx.remaining_accounts[current_creator_ata_index];

                        tokens_to_transfer.push([ata, creator_ata]);
                    } else if input_info.method == "burn" {
                        tokens_to_burn.push([mint, ata]);
                    }
                }
                _ => msg!("Token standard not found"),
            };
        }

        //Mint
        //if success burn or transfer
        msg!("SHOULD MINT NOW");

        // let output_indexes = parse_indexes(&input_indexer_json)?;

        // if tokens_to_transfer.len() > 0 {
        //     let transfer_atas = tokens_to_transfer
        //         .iter()
        //         .map(|&x| x[0].key().to_string())
        //         .collect::<Vec<_>>();

        //     //There should be no duplicates
        //     require!(
        //         has_unique_elements(&transfer_atas),
        //         TransmuterError::DuplicateInputAccount
        //     );

        //     //There should be the correct number of inputs
        //     require!(
        //         &transfer_atas.len() == &transmuter.inputs.len(),
        //         TransmuterError::InvalidInputAccount
        //     );

        //     //transfer nft to owner
        //     for index in 0..tokens_to_transfer.len() {
        //         let cpi_accounts = Transfer {
        //             from: tokens_to_transfer[index][0].to_account_info(),
        //             to: tokens_to_transfer[index][1].to_account_info(),
        //             authority: ctx.accounts.user.to_account_info(),
        //         };

        //         let cpi_program = ctx.accounts.token_program.to_account_info();
        //         transfer(CpiContext::new(cpi_program, cpi_accounts), 1)?;
        //     }
        // }

        // if tokens_to_burn.len() > 0 {
        //     let burn_atas = tokens_to_burn
        //         .iter()
        //         .map(|&x| x[0].key().to_string())
        //         .collect::<Vec<_>>();

        //     //There should be no duplicates
        //     require!(
        //         has_unique_elements(&burn_atas),
        //         TransmuterError::DuplicateInputAccount
        //     );

        //     //There should be the correct number of inputs
        //     require!(
        //         &burn_atas.len() == &transmuter.inputs.len(),
        //         TransmuterError::InvalidInputAccount
        //     );

        //     for index in 0..tokens_to_burn.len() {
        //         let cpi_accounts = Burn {
        //             mint: tokens_to_burn[index][0].to_account_info(),
        //             from: tokens_to_burn[index][1].to_account_info(),
        //             authority: ctx.accounts.user.to_account_info(),
        //         };

        //         let cpi_program = ctx.accounts.token_program.to_account_info();
        //         burn(CpiContext::new(cpi_program, cpi_accounts), 1)?;
        //     }
        // }

        msg!("AFTER");

        Ok(())
    }
}

//for titandogs

//-Burn 1 NFT
//-Generate 5 metadata + 5 images
//-Mint 5 generic NFT with metadata and images

//-Burn 5 NFTs
//-Mint 1 generic NFT with metadata + image

//should be able to generate image from input metadata or provided images

//rule part to titan => merge (merge all input nft into 1 output nft)
//-merge images to trait data
//-add all traits to metadata

//rule titan to part => split (1 trait = 1 nft)
//-for each trait find image in trait data and mint nft

//rule breeding => inherit (each output nft trait is randomly selected from inputs)
//-for each trait type list options
//-select a random one on list
//apply to output nft

//Should be a command to close transformer
