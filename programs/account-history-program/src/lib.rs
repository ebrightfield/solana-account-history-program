pub mod errors;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use errors::AccountHistoryProgramError;
use instructions::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod account_history_program {
    use super::*;

    pub fn initialize_account_history(
        ctx: Context<InitializeAccountHistory>,
        capacity: u32,
        data_regions: Vec<(u32, u32)>,
        min_slot_delay: Option<u32>,
        min_close_delay: Option<u32>,
    ) -> Result<()> {
        let crank_authority = ctx.remaining_accounts.get(0).map(|act_info| act_info.key());
        ctx.accounts.process(
            capacity,
            min_slot_delay.unwrap_or(1),
            min_close_delay.unwrap_or(0),
            data_regions,
            crank_authority,
        )?;
        Ok(())
    }

    pub fn update(ctx: Context<Update>) -> Result<()> {
        ctx.accounts.process()?;
        Ok(())
    }

    pub fn initialize_account_close(ctx: Context<InitializeAccountClose>) -> Result<()> {
        ctx.accounts.process()?;
        Ok(())
    }

    pub fn resolve_account_close(ctx: Context<ResolveAccountClose>) -> Result<()> {
        ctx.accounts.process()?;
        Ok(())
    }
}
