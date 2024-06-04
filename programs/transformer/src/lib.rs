use anchor_lang::prelude::*;
use anchor_spl::token::{burn, transfer, Burn, TokenAccount, Transfer};

use mpl_token_metadata::accounts::Metadata;

use url::Url;

mod contexts;
use contexts::*;

mod errors;
use errors::*;

mod structs;
use structs::*;

mod utils;
use utils::*;

// use spl_token::solana_program::program::invoke_signed;

declare_id!("H8SJKV7T4egtcwoA2HqSCNYeqrTJuA7SDSeZNrAgMmpf");

#[program]
pub mod transformer {
    use super::*;

    // Creator methods
    pub fn transmuter_create(
        ctx: Context<TransmuterCreate>,
        seed: u64,
        _input_length: u64,
        _output_length: u64,
        traits_uri: String,
    ) -> Result<()> {
        //Fee 0.75 SOL
        let _ = ctx
            .accounts
            .pay_fee(ctx.accounts.owner.to_account_info(), 75000000);

        //Fee 0.25 SOL
        let _ = ctx
            .accounts
            .pay_fee(ctx.accounts.wba.to_account_info(), 25000000);
        //if output rule is split, there could only be one input!
        //if output rule is merge, there could only be one output!

        let transmuter = &mut ctx.accounts.transmuter;
        transmuter.seed = seed;
        transmuter.creator = ctx.accounts.creator.as_ref().key();
        transmuter.auth_bump = ctx.bumps.auth;
        transmuter.transmuter_bump = ctx.bumps.transmuter;
        transmuter.traits_uri = traits_uri;

        Ok(())
    }

    pub fn transmuter_create_holder(
        ctx: Context<TransmuterCreateHolder>,
        seed: u64,
        _input_length: u64,
        _output_length: u64,
        traits_uri: String,
    ) -> Result<()> {
        let ata = &ctx.accounts.holder_ata.to_account_info();
        let mut ata_data: &[u8] = &ata.try_borrow_data()?;
        let deserialized_ata = TokenAccount::try_deserialize(&mut ata_data)?;

        msg!(
            "deserialized_ata.owner.key(): {:?}",
            deserialized_ata.owner.key()
        );
        msg!(
            "ctx.accounts.creator.key(): {:?}",
            ctx.accounts.creator.key()
        );
        msg!(
            "deserialized_ata.owner.key(): {:?}",
            deserialized_ata.owner.key()
        );

        require!(
            deserialized_ata.owner.key() == ctx.accounts.creator.key()
                && deserialized_ata.amount == 1,
            TransmuterError::InvalidNFTOwner
        );

        let metadata: Metadata =
            Metadata::try_from(&ctx.accounts.holder_metadata.to_account_info())?;
        let collection_pubkey = metadata.collection.unwrap().key;

        msg!("collection_pubkey: {:?}", collection_pubkey);
        //TODO fix this
        require!(
            collection_pubkey.key() == collection_pubkey.key(),
            TransmuterError::InvalidNFTOwner
        );

        let transmuter = &mut ctx.accounts.transmuter;
        transmuter.seed = seed;
        transmuter.creator = ctx.accounts.creator.as_ref().key();
        transmuter.auth_bump = ctx.bumps.auth;
        transmuter.transmuter_bump = ctx.bumps.transmuter;
        transmuter.traits_uri = traits_uri;

        Ok(())
    }

    pub fn transmuter_set_input(
        ctx: Context<TransmuterSet>,
        _seed: u64,
        input_json: String,
    ) -> Result<()> {
        let transmuter = &mut ctx.accounts.transmuter;
        transmuter.inputs.push(input_json);
        Ok(())
    }

    pub fn transmuter_set_output(
        ctx: Context<TransmuterSet>,
        _seed: u64,
        output_json: String,
    ) -> Result<()> {
        let transmuter = &mut ctx.accounts.transmuter;
        transmuter.outputs.push(output_json);
        Ok(())
    }

    pub fn transmuter_close(_ctx: Context<TransmuterClose>) -> Result<()> {
        // Prevent any new transmutation
        // Still allow ongoing ones
        msg!("THIS IS A TEST");
        Ok(())
    }

    // User methods
    pub fn user_send_input<'info>(
        ctx: Context<UserSendInput>,
        _seed: u64,
        vault_seed: u64,
    ) -> Result<()> {
        let transmuter = &ctx.accounts.transmuter;
        let transmuter_inputs = parse_json_vec::<InputInfo>(&transmuter.inputs)?;
        let transmuter_outputs = parse_json_vec::<OutputInfo>(&transmuter.outputs)?;

        require!(
            !&ctx.accounts.vault_auth.user_lock,
            TransmuterError::UserLock
        );

        let is_first_match = ctx.accounts.vault_auth.handled_inputs.len() == 0;
        if is_first_match {
            ctx.accounts.vault_auth.vault_auth_bump = ctx.bumps.vault_auth;
            // Vault auth info
            ctx.accounts.vault_auth.transmuter = transmuter.key();
            ctx.accounts.vault_auth.user = ctx.accounts.user.key();
            ctx.accounts.vault_auth.seed = vault_seed;

            //Init locks
            ctx.accounts.vault_auth.user_lock = false;
            ctx.accounts.vault_auth.creator_lock = true;

            //Init trackers
            ctx.accounts.vault_auth.handled_inputs =
                (0..transmuter_inputs.len()).map(|_| None).collect();
            ctx.accounts.vault_auth.input_uris =
                (0..transmuter_inputs.len()).map(|_| None).collect();
            ctx.accounts.vault_auth.handled_outputs =
                (0..transmuter_outputs.len()).map(|_| None).collect();
        }

        //Find an input_info match
        let mut is_match = false;
        for index in 0..transmuter_inputs.len() {
            if ctx.accounts.vault_auth.handled_inputs.len() > 0
                && ctx.accounts.vault_auth.handled_inputs[index].is_some()
            {
                msg!("Index {:?} already exist in vault_auth", index);
                continue;
            }

            is_match = is_matching(
                &ctx.accounts.metadata.to_account_info(),
                &transmuter_inputs[index],
            )?;

            if is_match {
                ctx.accounts.vault_auth.handled_inputs[index] = Some(ctx.accounts.mint.key());
                //TODO Maybe optional if split or merge
                let input_metadata: Metadata =
                    Metadata::try_from(&ctx.accounts.metadata.to_account_info())?;
                ctx.accounts.vault_auth.input_uris[index] = Some(input_metadata.uri);
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

    //WILL need a cancel_input

    pub fn user_claim_output<'info>(
        ctx: Context<UserClaimOutput>,
        _seed: u64,
        _vault_seed: u64,
    ) -> Result<()> {
        msg!("TRANSMUTE");
        //Will need to call that several time
        let transmuter = &ctx.accounts.transmuter;
        let transmuter_inputs = parse_json_vec::<InputInfo>(&transmuter.inputs)?;
        let transmuter_outputs = parse_json_vec::<OutputInfo>(&transmuter.outputs)?;
        let vault_auth = &ctx.accounts.vault_auth;

        let some_handled_inputs: Vec<Option<Pubkey>> = vault_auth
            .handled_inputs
            .clone()
            .into_iter()
            .filter(|handled_input| handled_input.is_some())
            .collect();

        let all_inputs_handled = some_handled_inputs.len() == transmuter_inputs.len();
        require!(all_inputs_handled, TransmuterError::InvalidInputLength);

        let some_handled_outputs: Vec<Option<Pubkey>> = vault_auth
            .handled_outputs
            .clone()
            .into_iter()
            .filter(|handled_output| handled_output.is_some())
            .collect();

        let all_outputs_handled = some_handled_outputs.len() == transmuter_outputs.len();
        require!(!all_outputs_handled, TransmuterError::IsComplete);

        //Find output to use
        for index in 0..transmuter_outputs.len() {
            if ctx.accounts.vault_auth.handled_outputs[index] != None {
                continue;
            }

            //handle output
            let output_info: &OutputInfo = &transmuter_outputs[index];
            let mut has_minted = false;

            if output_info.rule.is_some() {
                msg!("There is an output rule");
                let rule = output_info.rule.as_ref().unwrap();
                msg!("rule.name: {:?}", rule.name);
                let mint_info = output_info.mint.as_ref().unwrap();

                if rule.name == "split" {
                    msg!("Split rule");
                    // Should be only 1 input uri
                    let input_uri = vault_auth.input_uris[0].clone().unwrap();
                    msg!("input_uri: {:?}", input_uri);

                    let parsed_url = Url::parse(&input_uri).unwrap();
                    let hash_query: Vec<_> = parsed_url.query_pairs().into_owned().collect();

                    let split_traits: Vec<(String, String)> = hash_query
                        .clone()
                        .into_iter()
                        .filter(|(key, value)| {
                            rule.trait_types
                                .iter()
                                .any(|(trait_key, trait_value)| trait_key == key)
                        })
                        .collect();

                    msg!("split_traits 0: {:?}", split_traits[0]);

                    let mut mint_uri = mint_info.uri.to_owned() + "?";
                    msg!("use mint uri: {:?}", mint_uri);

                    for i in 0..split_traits.len() {
                        let key = &split_traits[i].0;
                        let value = &split_traits[i].1;

                        msg!("key: {:?}, val: {:?}", key, value);
                        mint_uri = mint_uri + key + "=" + value;

                        if i != split_traits.len() - 1 {
                            mint_uri = mint_uri + "&"
                        }
                    }

                    msg!("new mint uri: {:?}", mint_uri);

                    //TODO apply this to mint
                    let output_collection = &output_info.collection;

                    //mint as much as input traits (max output)
                    &ctx.accounts.mint_token();
                    &ctx.accounts.create_metadata(
                        &mint_info.title,
                        &mint_info.symbol,
                        &mint_uri,
                        &output_info.collection,
                        500,
                    );
                    &ctx.accounts.create_master_edition();
                    &ctx.accounts.update_authority();
                } else if rule.name == "merge" {
                    let mut trait_values: Vec<(String, String)> = Vec::new();
                    for (key, value) in rule.trait_types.clone().into_iter() {
                        for j in 0..transmuter_inputs.len() {
                            let input_uri = vault_auth.input_uris[j].clone().unwrap();

                            let parsed_url = Url::parse(&input_uri).unwrap();
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
                        .join("");

                    let uri = mint_info.uri.to_owned() + "?" + &uri_traits[0..uri_traits.len() - 1];

                    &ctx.accounts.mint_token();
                    &ctx.accounts.create_metadata(
                        &mint_info.title,
                        &mint_info.symbol,
                        &uri,
                        &output_info.collection,
                        500,
                    );
                    &ctx.accounts.create_master_edition();
                    &ctx.accounts.update_authority();
                } else {
                    msg!("Rule not found");
                }
                has_minted = true
            } else {
                //TODO ADD COLLECTION
                msg!("There is no rule");
                let mint_info = output_info.mint.as_ref().unwrap();
                let _ = ctx.accounts.mint_token();
                let _ = ctx.accounts.create_metadata(
                    &mint_info.title,
                    &mint_info.symbol,
                    &mint_info.uri,
                    &output_info.collection,
                    500,
                );
                let _ = ctx.accounts.create_master_edition();
                let _ = ctx.accounts.update_authority();
                has_minted = true
            }

            require!(has_minted, TransmuterError::MintFailed);
            ctx.accounts.vault_auth.handled_outputs[index] = Some(ctx.accounts.mint.key());
            break;
        }

        ctx.accounts.vault_auth.user_lock = true;
        ctx.accounts.vault_auth.creator_lock = false;

        Ok(())
    }

    pub fn creator_resolve_input<'info>(
        ctx: Context<CreatorResolveInput>,
        _seed: u64,
        vault_seed: u64,
    ) -> Result<()> {
        let transmuter = &ctx.accounts.transmuter;
        let transmuter_inputs = parse_json_vec::<InputInfo>(&transmuter.inputs)?;

        let vault_auth = &ctx.accounts.vault_auth;

        require!(!vault_auth.creator_lock, TransmuterError::NotClaimed);

        require!(
            vault_auth
                .handled_inputs
                .contains(&Some(ctx.accounts.mint.key())),
            TransmuterError::InvalidInputAccount
        );

        let input_info_index = vault_auth
            .handled_inputs
            .iter()
            .position(|&input: &Option<Pubkey>| input == Some(ctx.accounts.mint.key()))
            .unwrap();

        let input_info: &InputInfo = &transmuter_inputs[input_info_index];

        require!(
            input_info.method.as_str() == "transfer",
            TransmuterError::InvalidResolveMethod
        );

        let vault_seed_bytes = vault_seed.to_le_bytes();
        let seeds = &[
            b"vaultAuth",
            ctx.accounts.transmuter.to_account_info().key.as_ref(),
            ctx.accounts.user.to_account_info().key.as_ref(),
            &vault_seed_bytes.as_ref(),
            &[vault_auth.vault_auth_bump],
        ];
        let signer_seeds = &[&seeds[..]];

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

        Ok(())
    }

    pub fn creator_burn_input<'info>(
        ctx: Context<CreatorBurnInput>,
        _seed: u64,
        vault_seed: u64,
    ) -> Result<()> {
        let transmuter = &ctx.accounts.transmuter;
        let transmuter_inputs = parse_json_vec::<InputInfo>(&transmuter.inputs)?;
        let vault_auth = &ctx.accounts.vault_auth;

        require!(!vault_auth.creator_lock, TransmuterError::NotClaimed);

        require!(
            vault_auth
                .handled_inputs
                .contains(&Some(ctx.accounts.mint.key())),
            TransmuterError::InvalidInputAccount
        );

        let input_info_index = vault_auth
            .handled_inputs
            .iter()
            .position(|&input: &Option<Pubkey>| input == Some(ctx.accounts.mint.key()))
            .unwrap();

        let input_info: &InputInfo = &transmuter_inputs[input_info_index];

        require!(
            input_info.method.as_str() == "burn",
            TransmuterError::InvalidResolveMethod
        );

        let vault_seed_bytes = vault_seed.to_le_bytes();
        let seeds = &[
            b"vaultAuth",
            ctx.accounts.transmuter.to_account_info().key.as_ref(),
            ctx.accounts.user.to_account_info().key.as_ref(),
            &vault_seed_bytes.as_ref(),
            &[vault_auth.vault_auth_bump],
        ];
        let signer_seeds = &[&seeds[..]];

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

        Ok(())
    }
}