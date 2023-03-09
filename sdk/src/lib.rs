use bytemuck::{cast_slice, Pod};
use account_history_program::state::AccountHistoryHeader;
use crate::error::AccountHistorySdkError;

pub mod error;

pub struct AccountHistory<T: Pod> {
    header: AccountHistoryHeader,
    data: Vec<T>,
}

impl<T: Pod> AccountHistory<T> {
    pub fn from_buffer(data: &mut [u8]) -> error::Result<Self> {
        let history = account_history_program::state::AccountHistory::from_buffer(data)
            .map_err(|_| AccountHistorySdkError::AccountHistoryDeserializeFailure)?;
        Ok(AccountHistory {
            header: history.header(),
            data: cast_slice::<_, T>(&history.data()).to_vec(),
        })
    }
}