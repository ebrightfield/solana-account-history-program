use anchor_lang::prelude::*;
use crate::errors::AccountHistoryProgramError;
use crate::state::AccountHistoryRaw;


/// Push a new blob of data onto a history account.
#[derive(Accounts)]
pub struct Update<'info> {
    /// Signer performing the update
    signer: Signer<'info>,
    /// CHECK: The history account being updated
    #[account(mut)]
    account_state_history: UncheckedAccount<'info>,
    /// CHECK: The account's data type is not read by this program
    watched_account: UncheckedAccount<'info>,
}

impl<'info> Update<'info> {
    pub fn process(&mut self) -> Result<()> {
        let mut data = self.account_state_history.data.borrow_mut();
        let mut account_history = AccountHistoryRaw::from_buffer(&mut data)?;
        // Check we're indexing the correct account
        if self.watched_account.key() != account_history.header.associated_account {
            return err!(AccountHistoryProgramError::NotCorrectAccount);
        }
        // Check if the update authority matches (if update authority is not Default::default).
        if account_history.header.update_authority != Pubkey::default()
            && account_history.header.update_authority != self.signer.key() {
            return err!(AccountHistoryProgramError::NotUpdateAuthority);
        }
        // Try to push a new data snapshot
        let curr_slot = Clock::get()?.slot;
        account_history.push(
            &self.watched_account.data.borrow(),
            curr_slot,
        )?;
        Ok(())
    }
}
