use crate::errors::TransmuterError;
use crate::structs::rule::Rule;
use crate::structs::{Indexes, InputInfo, OutputInfo, TraitInfo};
use anchor_lang::error::Error;
use serde_json::Result as Result_serde;

pub fn parse_metadata(json: &String) -> Result<InputInfo, Error> {
    let result: Result_serde<InputInfo> = serde_json::from_str(json);
    let input_info = match result {
        Ok(input_info) => input_info,
        Err(_error) => panic!("{}", TransmuterError::JSONParseError),
    };
    Ok(input_info)
}

pub fn parse_input(json: &String) -> Result<InputInfo, Error> {
    let result: Result_serde<InputInfo> = serde_json::from_str(json);
    let input_info = match result {
        Ok(input_info) => input_info,
        Err(_error) => panic!("{}", TransmuterError::JSONParseError),
    };
    Ok(input_info)
}

pub fn parse_output(json: &String) -> Result<OutputInfo, Error> {
    let result: Result_serde<OutputInfo> = serde_json::from_str(json);
    let output_info = match result {
        Ok(output_info) => output_info,
        Err(_error) => panic!("{}", TransmuterError::JSONParseError),
    };
    Ok(output_info)
}

pub fn parse_indexes(json: &String) -> Result<Vec<Indexes>, Error> {
    let result: Result_serde<Vec<Indexes>> = serde_json::from_str(json);
    let indexes = match result {
        Ok(indexes) => indexes,
        Err(_error) => panic!("{}", TransmuterError::JSONParseError),
    };
    Ok(indexes)
}

pub fn parse_trait(json: &String) -> Result<TraitInfo, Error> {
    let result: Result_serde<TraitInfo> = serde_json::from_str(json);
    let trait_info = match result {
        Ok(trait_info) => trait_info,
        Err(_error) => panic!("{}", TransmuterError::JSONParseError),
    };
    Ok(trait_info)
}

pub fn parse_rule(json: &String) -> Result<Rule, Error> {
    let result: Result_serde<Rule> = serde_json::from_str(json);
    let rule = match result {
        Ok(rule) => rule,
        Err(_error) => panic!("{}", TransmuterError::JSONParseError),
    };
    Ok(rule)
}
