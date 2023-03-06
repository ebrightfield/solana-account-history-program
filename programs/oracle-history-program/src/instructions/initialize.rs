use anchor_lang::prelude::*;
use crate::state::OracleHistory;


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
    /// CHECK: The oracle's exact data type depends on oracle_history.oracle_type.
    #[account(
        address=oracle_history.load()?.associated_oracle()
    )]
    oracle: UncheckedAccount<'info>,
    system_program: Program<'info, System>,
}

impl<'info> InitializeOracleHistory<'info> {
    pub fn process(&mut self) -> Result<()> {
        let mut oracle_history = self.oracle_history.load_mut()?;
        oracle_history.associated_oracle = self.oracle.key();
        oracle_history.push(
            *self.oracle.key,
            &self.oracle.data.borrow()
        )?;
        Ok(())
    }
}