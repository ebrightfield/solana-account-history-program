use crate::errors::AccountHistoryProgramError;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Slot;
use bytemuck::{Pod, Zeroable};
use std::mem;
use std::num::NonZeroU64;
use std::ops::Index;

pub const ACCOUNT_HISTORY_TAG: u64 = 0;

/// Contains metadata like the account's capacity, element size,
/// number of updates, and locations of the account data being
/// recorded.
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct AccountHistoryHeader {
    account_tag: u64,
    /// The target account. Only historical data from this account will be indexed.
    pub(crate) associated_account: Pubkey,
    /// Only this account can close the history account and reclaim its rent lamports.
    pub(crate) close_authority: Pubkey,
    /// If not `Pubkey::default()`, only this address can sign for historical updates.
    pub(crate) update_authority: Pubkey,
    /// Total amount of space available for elements.
    pub(crate) capacity: u32,
    /// Includes the size of the slot.
    pub(crate) data_element_size: u32,
    /// Total number of updates that have executed.
    num_updates: u64,
    /// New data must be at least this many slots new compared to the
    /// most recently indexed data.
    pub(crate) min_slot_delay: u32,
    /// The account can be closed only after this delay.
    pub(crate) min_close_delay: u32,
    /// Zero when uninitialized. Slot number of when the history account
    /// close process was started.
    pub(crate) close_initiated: Option<NonZeroU64>,
    /// Collection of byte offsets and ranges from which to collect
    /// account state.
    /// Paired values, in the form of (offset, range). For example,
    /// (0,8) signifies the first 8 bytes. (48, 16) signifies
    /// a 16 byte span starting at the 48th byte.
    /// Maximum of 8 pairs of (offset, range), and they must
    /// be non-zero in length
    pub(crate) data_regions: [u32; 16],
}

impl Default for AccountHistoryHeader {
    fn default() -> Self {
        Self {
            account_tag: ACCOUNT_HISTORY_TAG,
            associated_account: Default::default(),
            close_authority: Default::default(),
            update_authority: Default::default(),
            capacity: 0,
            data_element_size: 0,
            num_updates: 0,
            min_slot_delay: 0,
            min_close_delay: 0,
            close_initiated: None,
            data_regions: [0; 16],
        }
    }
}

/// Data account, stores a data and a header.
#[derive(Debug)]
#[repr(C)]
pub struct AccountHistory<'data> {
    /// Metadata about the account history
    pub(crate) header: &'data mut AccountHistoryHeader,
    /// Historical account state
    data: &'data mut [u8],
}

impl<'data> AccountHistory<'data> {
    /// PDA generation just takes a random 32-byte seed.
    pub fn get_program_address(seed: [u8; 32]) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                seed.as_ref(),
            ],
            &crate::ID,
        )
    }

    /// Calculate the necessary size of an account history account
    /// with the given parameters.
    pub fn size_of(capacity: u32, data_locations: &[(u32, u32)]) -> usize {
        mem::size_of::<AccountHistoryHeader>()
            + capacity as usize
                * data_locations
                    .iter()
                    .map(|(_, len)| *len as usize)
                    .sum::<usize>()
    }

    /// Constructor.
    pub fn from_buffer(data: &'data mut [u8]) -> Result<Self> {
        let (header, data) = data.split_at_mut(mem::size_of::<AccountHistoryHeader>());
        let header = bytemuck::from_bytes_mut::<AccountHistoryHeader>(header);
        Ok(Self { header, data })
    }

    /// Most recently modified index. Returns zero when there is no data.
    pub fn most_recent_index(&self) -> usize {
        self.header.num_updates as usize % self.header.capacity as usize
    }

    /// Most recently added value. Returns zeroed bytes when there is no data.
    pub fn most_recent_entry(&self) -> &[u8] {
        let offset = self.most_recent_index() * self.header.data_element_size as usize;
        &self.data[offset..offset + self.header.data_element_size as usize]
    }

    /// The intended way to add a new element to this struct.
    /// Takes the current slot, and a reference to the account's data.
    ///
    /// Concatenates all the data regions being copied, prepends the slot value passed,
    /// and indexes the new data, replacing either an uninitialized or oldest value.
    ///
    /// You cannot add a value when this struct is being closed.
    /// This function also performs a minimum delay check on the passed slot number.
    pub fn push(&mut self, data: &[u8], slot: Slot) -> Result<()> {
        if self.header.close_initiated != None {
            return err!(AccountHistoryProgramError::AccountBeingClosed);
        }
        let last_slot = bytemuck::from_bytes::<Slot>(&self.most_recent_entry()[..8]);
        if *last_slot + self.header.min_slot_delay as u64 > slot {
            return err!(AccountHistoryProgramError::NotNewSlot);
        }
        // Obtain a mutable slice of the byte portion to be overwritten
        let mut offset = (self.header.num_updates as usize + 1) % self.header.capacity as usize;
        offset *= self.header.data_element_size as usize;
        let (_, from_offset) = self.data.split_at_mut(offset);
        let (buf, _) = from_offset.split_at_mut(self.header.data_element_size as usize);
        // Copy over the data
        let mut new_data = slot.to_le_bytes().to_vec();
        self.header
            .data_regions
            .chunks(2)
            .for_each(|val| {
                new_data.extend_from_slice(&data[val[0] as usize..(val[0] + val[1]) as usize]);
            });
        buf.copy_from_slice(&new_data);
        // Increment the counter that keeps track of indexing
        self.header.num_updates += 1;
        Ok(())
    }

    /// Total number of successful calls to `self.push`.
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

/// Iterates from newest value to oldest.
pub struct AccountHistoryIterator<'data> {
    val: &'data AccountHistory<'data>,
    counter: usize,
    index: usize,
}

impl<'data> Index<usize> for AccountHistory<'data> {
    type Output = [u8];
    fn index(&self, index: usize) -> &Self::Output {
        let index = index % self.header.capacity as usize;

        &self.data[index..index + self.header.data_element_size as usize]
    }
}

impl<'data> Iterator for AccountHistoryIterator<'data> {
    type Item = &'data [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.counter < self.val.len() {
            let slice =
                &self.val.data[self.index..self.index + self.val.header.data_element_size as usize];
            self.counter += 1;
            self.index = if self.index == 0 {
                self.val.capacity() - self.val.header.data_element_size as usize
            } else {
                self.index - self.val.header.data_element_size as usize
            };
            Some(slice)
        } else {
            None
        }
    }
}

impl<'data> From<&'data AccountHistory<'data>> for AccountHistoryIterator<'data> {
    fn from(value: &'data AccountHistory<'data>) -> Self {
        let start = value.most_recent_index() * value.header.data_element_size as usize;
        Self {
            val: &value,
            counter: 0,
            index: start,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default, Debug, Clone, Copy, Pod, Zeroable)]
    #[repr(C, align(8))]
    struct Price(u64, i64, u64);

    const ELEM_SIZE: usize = 24usize;
    const CAPACITY: usize = 5usize;

    #[test]
    fn iteration() {
        let key = Pubkey::new_unique();
        let mut header = AccountHistoryHeader {
            associated_account: key,
            capacity: CAPACITY as u32,
            data_element_size: ELEM_SIZE as u32,
            ..Default::default()
        };
        header.data_regions[1] = 8;
        header.data_regions[2] = 16;
        header.data_regions[3] = 8;
        let header_bytes = bytemuck::bytes_of(&header);
        let mut mock_data = [header_bytes, &[0u8; CAPACITY * ELEM_SIZE]].concat();
        let mut vec = AccountHistory::from_buffer(&mut mock_data).unwrap();
        let mock_act_data = [10u64.to_le_bytes(), [0u8; 8], 20u64.to_le_bytes(), [0u8; 8]].concat();
        vec.push(&mock_act_data, 1).unwrap();
        let mock_act_data = [11u64.to_le_bytes(), [0u8; 8], 22u64.to_le_bytes(), [0u8; 8]].concat();
        vec.push(&mock_act_data, 2).unwrap();
        let mock_act_data = [12u64.to_le_bytes(), [0u8; 8], 24u64.to_le_bytes(), [0u8; 8]].concat();
        vec.push(&mock_act_data, 3).unwrap();
        let mock_act_data = [13u64.to_le_bytes(), [0u8; 8], 26u64.to_le_bytes(), [0u8; 8]].concat();
        vec.push(&mock_act_data, 4).unwrap();
        // len check
        assert_eq!(4, vec.len());
        println!("{:?}", &vec.most_recent_entry());
        println!("{:?}", &vec.most_recent_index());
        println!("{:?}", &vec.data);

        let mut j = 13i64;
        assert_eq!(mem::size_of::<Price>(), 24);
        assert_eq!(mem::align_of::<Price>(), 8);
        for i in AccountHistoryIterator::from(&vec) {
            println!("{:?}", &i);
            let price = bytemuck::from_bytes::<Price>(i);
            assert_eq!(price.1, j);
            assert_eq!(price.2, j as u64 * 2);
            j -= 1;
        }
        // // Should not iterate through anything
        // let vec = StackVecModulo::<u64, 5>::default();
        // let mut j = 0u64;
        // for _ in StackVecModuloIterator::from(&vec) {
        //     j += 1;
        // }
        // assert_eq!(j, 0);
    }

    #[test]
    fn push() {
        // let mut vec = StackVecModulo::<u64, 5>::default();
        // // len check
        // assert_eq!(0, vec.len());
        // assert_eq!(*vec.most_recent_entry(), 0);
        // vec.push(0);
        // assert_eq!(*vec.most_recent_entry(), 0);
        // vec.push(2);
        // assert_eq!(*vec.most_recent_entry(), 2);
        // vec.push(4);
        // vec.push(6);
        // vec.push(8);
        // assert_eq!(*vec.most_recent_entry(), 8);
        // vec.push(10);
        // assert_eq!(*vec.most_recent_entry(), 10);
        // vec.push(12);
        // // len check should now return 5
        // assert_eq!(5, vec.len());
        //
        // // Should always return last pushed value
        // assert_eq!(*vec.most_recent_entry(), 12);
    }
}
