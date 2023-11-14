use crate::errors::TransmuterError;
use crate::structs::{InputIndex, InputInfo, OutputInfo};
use anchor_lang::error::Error;
use serde_json::Result as Result_serde;

pub fn parse_metadata(json: &String) -> Result<InputInfo, Error> {
    let input_info_result: Result_serde<InputInfo> = serde_json::from_str(json);
    let input_info = match input_info_result {
        Ok(input_info) => input_info,
        Err(_error) => panic!("{}", TransmuterError::JSONParseError),
    };
    Ok(input_info)
}

pub fn parse_input(json: &String) -> Result<InputInfo, Error> {
    let input_info_result: Result_serde<InputInfo> = serde_json::from_str(json);
    let input_info = match input_info_result {
        Ok(input_info) => input_info,
        Err(_error) => panic!("{}", TransmuterError::JSONParseError),
    };
    Ok(input_info)
}

pub fn parse_output(json: &String) -> Result<OutputInfo, Error> {
    let output_info_result: Result_serde<OutputInfo> = serde_json::from_str(json);
    let output_info = match output_info_result {
        Ok(output_info) => output_info,
        Err(_error) => panic!("{}", TransmuterError::JSONParseError),
    };
    Ok(output_info)
}

pub fn parse_strings(json: &String) -> Result<Vec<String>, Error> {
    let result: Result_serde<Vec<String>> = serde_json::from_str(json);
    let strings = match result {
        Ok(strings) => strings,
        Err(_error) => panic!("{}", TransmuterError::JSONParseError),
    };
    Ok(strings)
}

pub fn parse_indexes(json: &String) -> Result<Vec<InputIndex>, Error> {
    let result: Result_serde<Vec<InputIndex>> = serde_json::from_str(json);
    let indexes = match result {
        Ok(indexes) => indexes,
        Err(_error) => panic!("{}", TransmuterError::JSONParseError),
    };
    Ok(indexes)
}
