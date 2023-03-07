use anchor_lang::prelude::*;
use crate::errors::AccountHistoryProgramError;
use crate::state::AccountHistory;


/// Push a new blob of data onto a history account.
#[derive(Accounts)]
pub struct Update<'info> {
    /// Signer performing the update
    signer: Signer<'info>,
    /// The history account being updated
    #[account(mut)]
    account_state_history: UncheckedAccount<'info>,
    /// CHECK: The account's data type is not read by this program
    watched_account: UncheckedAccount<'info>,
}

impl<'info> Update<'info> {
    pub fn process(&mut self) -> Result<()> {
        let mut data = self.account_state_history.data.borrow_mut();
        let mut oracle_history = AccountHistory::from_buffer(&mut data)?;
        if oracle_history.header.update_authority != Pubkey::default()
            && oracle_history.header.update_authority != self.signer.key() {
            return err!(AccountHistoryProgramError::NotUpdateAuthority);
        }
        let curr_slot = Clock::get()?.slot;
        assert_eq!(
            self.watched_account.key(),
            oracle_history.header.associated_account,
        );
        oracle_history.push(
            &self.watched_account.data.borrow(),
            curr_slot,
        )?;
        Ok(())
    }
}