use solana_sdk::pubkey::Pubkey;
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum AccountHistorySdkError {
    #[error("Account not found: {0}")]
    AccountNotFound(Pubkey),

    #[error("Could not deserialize account history")]
    AccountHistoryDeserializeFailure,

    #[error("The mint {0} is not supported")]
    UnsupportedMint(Pubkey),

    #[error("Invalid account size for {0}, expect: {1}, got: {2}")]
    InvalidAccountSize(Pubkey, usize, usize),
}

pub type Result<T> = std::result::Result<T, AccountHistorySdkError>;
