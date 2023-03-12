pub mod config;
use anchor_lang::prelude::*;
use anchor_lang::InstructionData;
use anyhow::{anyhow, Result};
use solana_sdk::instruction::Instruction;
use account_history_program::state::account_history_address;


/// Represents a contiguous chunk of bytes.
/// Expressed as "o:l" where o is the byte offset where the region begins,
/// and l is the region's length.
/// For example, 0:3 is expresses the slice `data[0..3]`.
#[derive(Debug)]
pub struct DataRegion(u32, u32);


impl TryFrom<&str> for DataRegion {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (offset, range) = value.rsplit_once(":")
            .ok_or(anyhow!("Invalid data region,\
            must be offset:range where offset and range are positive integers"))?;
        let offset: u32 = offset.parse()
            .map_err(|_| anyhow!("invalid integer value for offset: {}", offset))?;
        let range: u32 = range.parse()
            .map_err(|_| anyhow!("invalid integer value for range: {}", range))?;
        Ok(Self(offset, range))
    }
}

impl Into<(u32, u32)> for DataRegion {
    fn into(self) -> (u32, u32) {
        (self.0, self.1)
    }
}


pub fn initialize_account_history(
    capacity: u32,
    data_regions: Vec<(u32, u32)>,
    min_slot_delay: Option<u32>,
    min_close_delay: Option<u32>,
    payer: Pubkey,
    seed: Pubkey,
    watched_account: Pubkey,
) -> Instruction {
    let data = account_history_program::instruction::InitializeAccountHistory {
        capacity,
        data_regions,
        min_slot_delay,
        min_close_delay,
    }.data();
    let history_pubkey = account_history_address(seed.to_bytes()).0;
    let accounts = account_history_program::accounts::InitializeAccountHistory {
        payer,
        seed,
        account_state_history: history_pubkey,
        watched_account,
        system_program: System::id(),
    }.to_account_metas(None);
    Instruction {
        data,
        accounts,
        program_id: account_history_program::id(),
    }
}

pub fn update(
    signer: Pubkey,
    account_history: Pubkey,
    watched_account: Pubkey,
) -> Instruction {
    let data = account_history_program::instruction::Update.data();
    let accounts = account_history_program::accounts::Update {
        signer,
        account_state_history: account_history,
        watched_account,
    }.to_account_metas(None);
    Instruction {
        data,
        accounts,
        program_id: account_history_program::id(),
    }
}

pub fn initialize_account_close(
    close_authority: Pubkey,
    account_history: Pubkey,
) -> Instruction {
    let data = account_history_program::instruction::InitializeAccountClose.data();
    let accounts = account_history_program::accounts::InitializeAccountClose {
        close_authority,
        account_state_history: account_history,
    }.to_account_metas(None);
    Instruction {
        data,
        accounts,
        program_id: account_history_program::id(),
    }
}

pub fn resolve_account_close(
    close_authority: Pubkey,
    account_history: Pubkey,
    rent_recipient: Option<Pubkey>,
) -> Instruction {
    let data = account_history_program::instruction::ResolveAccountClose.data();
    let rent_recipient = rent_recipient.unwrap_or(close_authority);
    let accounts = account_history_program::accounts::ResolveAccountClose {
        close_authority,
        rent_recipient,
        account_state_history: account_history,
    }.to_account_metas(None);
    Instruction {
        data,
        accounts,
        program_id: account_history_program::id(),
    }
}
