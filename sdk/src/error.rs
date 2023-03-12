use solana_sdk::pubkey::Pubkey;
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum AccountHistorySdkError {
    #[error("Account not found: {0}")]
    AccountNotFound(Pubkey),
}

pub type Result<T> = std::result::Result<T, AccountHistorySdkError>;
