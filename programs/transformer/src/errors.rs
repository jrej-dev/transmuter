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
    #[msg("Invalid resolve method provided for input")]
    InvalidResolveMethod,
    #[msg("Duplicate accounts provided for input")]
    DuplicateInputAccount,
    #[msg("Invalid NFT owner provided for the transmuter")]
    InvalidNFTOwner,
    #[msg("Invalid NFT owner provided for the transmuter")]
    InvalidInputLength,
    #[msg("Transmuter locked for user")]
    UserLock,
    #[msg("Transmuter locked for creator")]
    CreatorLock,
    #[msg("Transmutation complete")]
    IsComplete,
    #[msg("First output was not claimed")]
    NotClaimed,
    #[msg("Failed the minting process")]
    MintFailed,
    #[msg("input length provided not matching input required")]
    InvalidProgramOwner,
    #[msg("Rule could not be applied on transmute")]
    RuleNotApplied,
    #[msg("Missing trait in transmuter")]
    MissingTrait,
}
