use anchor_lang::prelude::*;
use anchor_lang::InstructionData;
use solana_sdk::instruction::Instruction;
use account_history_program::state::{account_history_address, AccountHistoryRaw};

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