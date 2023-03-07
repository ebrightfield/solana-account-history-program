use anchor_lang::prelude::*;
use crate::errors::AccountHistoryProgramError;
use crate::state::{AccountHistory, IntoBytes};

/// Create a new historical data account, configured
/// to watch a certain account.
#[derive(Accounts)]
#[instruction(capacity: u32, data_regions: Vec<(u32, u32)>)]
pub struct InitializeAccountHistory<'info> {
    /// Funds rent for the new oracle account.
    #[account(mut)]
    payer: Signer<'info>,
    /// CHECK: The history account being created.
    #[account(
        init,
        payer=payer,
        space=AccountHistory::size_of(capacity, &data_regions),
        seeds=[
            watched_account.key().as_ref(),
            capacity.to_le_bytes().as_ref(),
            data_regions.into_bytes().as_ref(),
        ],
        bump,
    )]
    account_state_history: UncheckedAccount<'info>,
    /// CHECK: The account's data type is not read by this program
    watched_account: UncheckedAccount<'info>,
    system_program: Program<'info, System>,
}

impl<'info> InitializeAccountHistory<'info> {
    pub fn process(&mut self, capacity: u32, min_slot_delay: u32, min_close_delay: u32, data_regions: Vec<(u32, u32)>, update_authority: Option<Pubkey>) -> Result<()> {
        let mut data = self.account_state_history.data.borrow_mut();
        let mut act_history = AccountHistory::from_buffer(&mut data)?;
        act_history.header.associated_account = self.watched_account.key();
        act_history.header.close_authority = self.payer.key();
        act_history.header.update_authority = update_authority.unwrap_or(Pubkey::default());
        act_history.header.capacity = capacity;
        act_history.header.min_slot_delay = min_slot_delay;
        act_history.header.min_close_delay = min_close_delay;
        // 8 bytes for slot, then data.
        act_history.header.data_element_size = 8u32 +
            data_regions.iter().map(|(_, i)| *i).sum::<u32>();
        act_history.header.data_regions = sanitize_data_regions(&data_regions)?;
        act_history.push(
            &self.watched_account.data.borrow(),
            Clock::get()?.slot,
        )?;
        Ok(())
    }
}

pub fn sanitize_data_regions(pairs: &[(u32, u32)]) -> Result<[u32; 16]> {
    let mut loc = [0u32; 16];
    let mut index_so_far = 0u32;
    for (i, pair) in pairs.iter().enumerate() {
        // No zero-length regions
        if pair.1 == 0 {
            return err!(AccountHistoryProgramError::InvalidDataRegions);
        }
        // We should always be ascending index values, and it should not be contiguous, so `>=`.
        // The case where both values are zero is a special initial case.
        if (pair.0 != 0 || index_so_far != 0) &&
            index_so_far >= pair.0 {
            return err!(AccountHistoryProgramError::InvalidDataRegions);
        }
        index_so_far = pair.0 + pair.1;
        loc[i] = pair.0;
        loc[i+1] = pair.1;
    }
    Ok(loc)
}