use anchor_lang::error_code;

#[error_code]
pub enum OracleHistoryProgramError {
    #[msg("Invalid Oracle Address")]
    InvalidOracleAddress,
    #[msg("Invalid Pyth Oracle Data")]
    InvalidOracleDataPyth,
    #[msg("Cannot add price, not a higher slot number")]
    NotNewSlot,
    #[msg("Oracle History already initialized")]
    AlreadyInitialized,
}
