use anchor_lang::prelude::*;
use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;
use anchor_spl::token::{burn, transfer, Burn, InitializeAccount, TokenAccount, Transfer};
use mpl_token_metadata::state::{Metadata, TokenMetadataAccount};
use std::str::FromStr;

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

    pub fn add_collection(
        ctx: Context<AddCollection>,
        seed: u64,
        candy_machine_json: String, /*{}*/
    ) -> Result<()> {
        //candy machine id

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
        let input_indexes = parse_indexes(&input_indexer_json)?;
        let mut tokens_to_transfer: Vec<[&AccountInfo<'_>; 2]> = Vec::new();
        let mut tokens_to_burn: Vec<[&AccountInfo<'_>; 2]> = Vec::new();

        for index in 0..transmuter.inputs.len() {
            let input_info = parse_input(&transmuter.inputs[index])?;
            msg!("token_standard, {}", input_info.token_standard);
            msg!("method, {}", input_info.method);
            msg!("collection, {}", input_info.collection);

            let current_input_indexes = &input_indexes[index];

            match input_info.token_standard.as_str() {
                "nft" => {
                    let metadata: Metadata = Metadata::from_account_info(
                        &ctx.remaining_accounts[current_input_indexes.metadata].to_account_info(),
                    )?;
                    let collection_pubkey = metadata.collection.unwrap().key;
                    msg!("collection pubkey: {:?}", collection_pubkey);

                    require!(
                        collection_pubkey.to_string() == input_info.collection,
                        TransmuterError::InvalidInputAccount
                    );

                    let current_ata_index = current_input_indexes.ata.unwrap();
                    let current_creator_ata_index = current_input_indexes.creator_ata.unwrap();

                    let ata: &AccountInfo<'_> = &ctx.remaining_accounts[current_ata_index];
                    let creator_ata: &AccountInfo<'_> =
                        &ctx.remaining_accounts[current_creator_ata_index];

                    let mint: &AccountInfo<'_> =
                        &ctx.remaining_accounts[current_input_indexes.mint];
                    msg!("ata : {}", ata.key());

                    if input_info.method == "transfer" {
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

        let output_indexes = parse_indexes(&input_indexer_json)?;

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
                &transfer_atas.len() == &transmuter.inputs.len(),
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
                &burn_atas.len() == &transmuter.inputs.len(),
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

        msg!("AFTER");

        Ok(())
    }
}

//for titandogs

//-Burn 1 NFT
//-Mint 5 generic NFT
//-Generate 5 metadata + 5 images
//-Update metadata of 5 NFTs

//-Burn 5 NFTs
//-Mint 1 generic NFT
//-Generate 1 metadata + image
//-Update metadata

//Should be a command to change owner from program to creator (closing transformer mint)

//should be able to generate image from input metadata
//rule part to titan => merge (merge all input nft into 1 output nft)
//rule titan to part => split (1 trait = 1 nft)
//rule breeding => inherit (each output nft trait is randomly selected from inputs)
