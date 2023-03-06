pub mod state;
pub mod svec_modulo;
pub mod errors;
pub mod instructions;

use anchor_lang::prelude::*;

use instructions::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod oracle_history_program {
    use super::*;

    pub fn initialize_oracle_history(ctx: Context<InitializeOracleHistory>) -> Result<()> {
        ctx.accounts.process()?;
        Ok(())
    }

    pub fn crank(ctx: Context<Crank>) -> Result<()> {
        ctx.accounts.process()?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
