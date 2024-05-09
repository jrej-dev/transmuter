use anchor_lang::error_code;

#[error_code]
pub enum TransmuterError {
    #[msg("Unable to pay transmuter creation fee")]
    CreationFeeError,
    #[msg("Unable to get auth bump")]
    AuthBumpError,
    #[msg("Unable to get transmuter bump")]
    TransmuterBumpError,
    #[msg("Unable to get vault bump")]
    VaultBumpError,
    #[msg("Unable to parse JSON content")]
    JSONParseError,
    #[msg("Invalid account provided for input")]
    InvalidInputAccount,
    #[msg("Duplicate accounts provided for input")]
    DuplicateInputAccount,
    #[msg("Invalid NFT owner provided for the transmuter")]
    InvalidNFTOwner,
    #[msg("Invalid NFT owner provided for the transmuter")]
    InvalidInputLength,
    #[msg("Transmutation complete")]
    TransmutationComplete,
    #[msg("Transmutation incomplete")]
    TransmutationIncomplete,
    #[msg("input length provided not matching input required")]
    InvalidProgramOwner,
    #[msg("Rule could not be applied on transmute")]
    RuleNotApplied,
    #[msg("Missing trait in transmuter")]
    MissingTrait,
}
