use anchor_lang::prelude::*;
use crate::state::OracleHistory;


/// Push a new oracle price onto its history account.
#[derive(Accounts)]
pub struct Crank<'info> {
    /// The oracle history account being updated
    #[account(
        mut,
        seeds=[
            oracle.key().as_ref(),
        ],
        bump,
    )]
    oracle_history: AccountLoader<'info, OracleHistory>,
    /// CHECK: The oracle's exact data type depends on oracle_history.oracle_type.
    #[account(
        address=oracle_history.load()?.associated_oracle()
    )]
    oracle: UncheckedAccount<'info>,
}

impl<'info> Crank<'info> {
    pub fn process(&mut self) -> Result<()> {
        self.oracle_history.load_mut()?.push(
            *self.oracle.key,
            &self.oracle.data.borrow()
        )?;
        Ok(())
    }
}