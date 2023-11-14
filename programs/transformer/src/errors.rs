use anchor_lang::error_code;

#[error_code]
pub enum TransmuterError {
    #[msg("Unable to get auth bump")]
    AuthBumpError,
    #[msg("Unable to get transmuter bump")]
    TransmuterBumpError,
    #[msg("Unable to parse JSON content")]
    JSONParseError,
    #[msg("Invalid account provided for input")]
    InvalidInputAccount,
    #[msg("Duplicate accounts provided for input")]
    DuplicateInputAccount,
}
