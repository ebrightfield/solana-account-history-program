use anchor_lang::prelude::*;
use std::mem;
use std::ops::Index;
use anchor_lang::solana_program::pubkey::Pubkey;
use bytemuck::Pod;
use crate::state::{ACCOUNT_HISTORY_TAG, AccountHistoryHeader};
use crate::errors::AccountHistoryProgramError;

/// Data account, stores a data and a header.
#[derive(Debug)]
#[repr(C)]
pub struct AccountHistory<'data, T: Pod> {
    /// Metadata about the account history
    pub(crate) header: &'data mut AccountHistoryHeader,
    /// Historical account state
    data: &'data mut [T],
}

impl<'data, T: Pod> AccountHistory<'data, T> {
    /// Constructor.
    pub fn from_buffer(data: &'data mut [u8]) -> Result<Self> {
        let (header, data) = data.split_at_mut(mem::size_of::<AccountHistoryHeader>());
        let header = bytemuck::from_bytes_mut::<AccountHistoryHeader>(header);
        if header.account_tag != ACCOUNT_HISTORY_TAG {
            return err!(AccountHistoryProgramError::InvalidAccountTag);
        }
        let data = bytemuck::try_cast_slice_mut::<_, T>(data)
            .map_err(|_| AccountHistoryProgramError::InvalidDataType)?;
        Ok(Self { header, data })
    }

    pub fn header(&self) -> AccountHistoryHeader {
        *self.header
    }

    pub fn data(&self) -> Vec<T> {
        self.data.to_vec()
    }

    /// Most recently modified index. Returns zero when there is no data.
    pub fn most_recent_index(&self) -> usize {
        self.header.num_updates as usize % self.header.capacity as usize
    }

    /// Most recently added value. Returns zeroed bytes when there is no data.
    pub fn most_recent_entry(&self) -> &T {
        &self.data[self.most_recent_index()]
    }

    /// Total number of element updates that have taken place on this history account.
    pub fn num_updates(&self) -> usize {
        self.header.num_updates as usize
    }

    /// The account whose state is being recorded on this history account.
    pub fn associated_account(&self) -> Pubkey {
        self.header.associated_account
    }

    /// Maximum number of elements supported by this account.
    pub fn capacity(&self) -> usize {
        self.header.capacity as usize
    }

    /// The number of values indexed so far. Ranges from 0 to `self.header.capacity`.
    pub fn len(&self) -> usize {
        std::cmp::min(
            self.header.num_updates as usize,
            self.header.capacity as usize,
        )
    }
}

impl<'data, T: Pod> Index<usize> for AccountHistory<'data, T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        let index = index % self.header.capacity as usize;
        &self.data[index]
    }
}

/// Iterates from newest value to oldest.
pub struct AccountHistoryIterator<'data, T: Pod> {
    inner: &'data AccountHistory<'data, T>,
    counter: usize,
    index: usize,
}

impl<'data, T: Pod> Iterator for AccountHistoryIterator<'data, T> {
    type Item = &'data T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.counter < self.inner.len() {
            let slice =
                &self.inner.data[self.index];
            self.counter += 1;
            self.index = if self.index == 0 {
                self.inner.capacity() - 1
            } else {
                self.index - 1
            };
            Some(slice)
        } else {
            None
        }
    }
}

impl<'data, T: Pod> From<&'data AccountHistory<'data, T>> for AccountHistoryIterator<'data, T> {
    fn from(value: &'data AccountHistory<'data, T>) -> Self {
        let start = value.most_recent_index() * value.header.data_element_size as usize;
        Self {
            inner: &value,
            counter: 0,
            index: start,
        }
    }
}
