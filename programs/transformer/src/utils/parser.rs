use crate::errors::TransmuterError;
use crate::structs::rule::Rule;
use crate::structs::{Indexes, InputInfo, OutputInfo, TraitInfo};
use crate::MintInfo;
use anchor_lang::error::Error;
use serde::Deserialize;
use serde_json::Result as Result_serde;
use spl_token::solana_program::msg;

pub fn parse_json<'a, T>(json: &'a String) -> Result<Vec<T>, Error>
where
    T: Deserialize<'a>,
{
    Ok(serde_json::from_str::<Vec<T>>(json).unwrap())
}