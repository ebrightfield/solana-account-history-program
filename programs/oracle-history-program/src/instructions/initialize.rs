use anchor_lang::prelude::*;
use crate::state::{OracleHistory, OracleType};


#[derive(Accounts)]
pub struct InitializeOracleHistory<'info> {
    /// Funds rent for the new oracle account.
    #[account(mut)]
    payer: Signer<'info>,
    /// The oracle history account being created.
    #[account(
        init,
        payer=payer,
        space=std::mem::size_of::<OracleHistory>(),
        seeds=[
            oracle.key().as_ref(),
        ],
        bump,
    )]
    oracle_history: AccountLoader<'info, OracleHistory>,
    /// CHECK: The oracle's expected type depends on `oracle_history.oracle_type`.
    oracle: UncheckedAccount<'info>,
    system_program: Program<'info, System>,
}

impl<'info> InitializeOracleHistory<'info> {
    pub fn process(&mut self, oracle_type: OracleType) -> Result<()> {
        let mut oracle_history = self.oracle_history.load_mut()?;
        oracle_history.oracle_type = oracle_type;
        oracle_history.associated_oracle = self.oracle.key();
        oracle_history.push(
            *self.oracle.key,
            &self.oracle.data.borrow()
        )?;
        Ok(())
    }
}