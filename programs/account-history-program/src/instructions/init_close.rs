use anchor_lang::prelude::*;
use crate::errors::AccountHistoryProgramError;
use crate::state::AccountHistoryRaw;

#[derive(Accounts)]
#[instruction(capacity: u32, data_regions: Vec<(u32, u32)>)]
pub struct InitializeAccountClose<'info> {
    /// The sole authority capable of closing the history account.
    #[account(mut)]
    close_authority: Signer<'info>,
    /// CHECK: The history account being created.
    #[account(mut)]
    account_state_history: UncheckedAccount<'info>,
}

impl<'info> InitializeAccountClose<'info> {
    pub fn process(&mut self) -> Result<()> {
        let mut data = self.account_state_history.data.borrow_mut();
        let mut act_history = AccountHistoryRaw::from_buffer(&mut data)?;
        if self.close_authority.key() != act_history.header.close_authority {
            return err!(AccountHistoryProgramError::NotCloseAuthority);
        }
        act_history.header.close_initiated = Some(Clock::get()?.slot.try_into().unwrap());
        Ok(())
    }
}