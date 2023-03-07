use anchor_lang::error_code;

#[error_code]
pub enum AccountHistoryProgramError {
    #[msg("Data regions are not well-ordered, non-zero, and non-contiguous")]
    InvalidDataRegions,
    #[msg("Cannot add data, not a high enough slot number")]
    NotNewSlot,
    #[msg("Signer provided does not match the update authority")]
    NotUpdateAuthority,
    #[msg("Cannot add data, this account is flagged to be closed")]
    AccountBeingClosed,
    #[msg("Cannot close account, wait period has not elapsed")]
    CannotCloseYet,
}
