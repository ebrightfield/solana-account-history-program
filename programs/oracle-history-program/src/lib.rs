pub mod state;
pub mod svec_modulo;
pub mod errors;
pub mod instructions;

use anchor_lang::prelude::*;

use instructions::*;
use state::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod oracle_history_program {
    use super::*;

    pub fn initialize_oracle_history(ctx: Context<InitializeOracleHistory>, oracle_type: OracleType) -> Result<()> {
        ctx.accounts.process(oracle_type)?;
        Ok(())
    }

    pub fn crank(ctx: Context<Crank>) -> Result<()> {
        ctx.accounts.process()?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
