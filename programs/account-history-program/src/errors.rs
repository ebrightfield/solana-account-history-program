use anchor_lang::error_code;

#[error_code]
pub enum AccountHistoryProgramError {
    #[msg("Cannot update historical data, invalid account")]
    NotCorrectAccount,
    #[msg("Data regions must have non-zero length")]
    InvalidDataRegions,
    #[msg("Cannot add data, not a high enough slot number")]
    NotNewSlot,
    #[msg("Signer provided does not match the update authority")]
    NotUpdateAuthority,
    #[msg("Signer provided does not match the close authority")]
    NotCloseAuthority,
    #[msg("Cannot add data, this account is flagged to be closed")]
    AccountBeingClosed,
    #[msg("Cannot close account, wait period has not elapsed")]
    CannotCloseYet,
    #[msg("Cannot close account, close process not yet initiated")]
    CloseNotInitiated,
}
