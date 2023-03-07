use anchor_lang::prelude::*;
use crate::errors::AccountHistoryProgramError;
use crate::state::AccountHistory;

#[derive(Accounts)]
pub struct ResolveAccountClose<'info> {
    /// The sole authority capable of closing the history account.
    #[account(mut)]
    close_authority: Signer<'info>,
    /// CHECK: Recipient of lamport rent
    #[account(mut)]
    rent_recipient: SystemAccount<'info>,
    /// CHECK: The history account being closed.
    #[account(mut)]
    account_state_history: UncheckedAccount<'info>,
}

impl<'info> ResolveAccountClose<'info> {
    pub fn process(&mut self) -> Result<()> {
        let mut data = self.account_state_history.data.borrow_mut();
        let act_history = AccountHistory::from_buffer(&mut data)?;
        // Check close authority
        if self.close_authority.key() != act_history.header.close_authority {
            return err!(AccountHistoryProgramError::NotCloseAuthority);
        }
        // Check readiness to close the account
        match act_history.header.close_initiated {
            Some(slot) => {
              if (u64::try_from(slot).unwrap() + act_history.header.min_close_delay as u64) > Clock::get()?.slot {
                  return err!(AccountHistoryProgramError::CannotCloseYet);
              }
            },
            None => {
                return err!(AccountHistoryProgramError::CloseNotInitiated);
            }
        }
        // Close account (lamports, owner, realloc)
        let dest_starting_lamports = self.rent_recipient.lamports();
        **self.rent_recipient.lamports.borrow_mut() =
            dest_starting_lamports.checked_add(self.account_state_history.lamports()).unwrap();
        **self.account_state_history.lamports.borrow_mut() = 0;

        self.account_state_history.assign(&System::id());
        self.account_state_history.realloc(0, false)?;

        Ok(())
    }
}