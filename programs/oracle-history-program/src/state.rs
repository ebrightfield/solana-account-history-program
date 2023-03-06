use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Slot;
use pyth_sdk_solana::state::load_price_account;
use crate::errors::OracleHistoryProgramError;
use crate::svec_modulo::{StackVecModulo, StackVecModuloIterator};

/// Sets the size of oracle price history accounts.
///
/// Cannot be larger than `(10240 - header) / mem::sizeof::<Price>()`.
const ORACLE_HISTORY_SIZE: usize = 100;

/// A record of a price at a given time..
/// The purpose of oracle history accounts is to store this data.
///
/// If you wanted to target some other kind of oracle data,
/// you'd do so by replacing this struct with some other data type.
#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
pub struct Price {
    pub price: i64,
    pub slot: Slot,
}

/// Determines how to deserialize the associated oracle account
/// when indexing.
#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
pub enum OracleType {
    #[default]
    Pyth,
}

/// Data account, stores oracle price data and a header that describes
/// the target oracle and a data type enum which tells how to deserialize the data.
#[account(zero_copy)]
#[repr(C)]
pub struct OracleHistory {
    associated_oracle: Pubkey,
    oracle_type: OracleType,
    _padding0: [u8; 5],
    prices: StackVecModulo<Price, ORACLE_HISTORY_SIZE>,
}

impl OracleHistory {
    pub const PRICES_LEN: usize = ORACLE_HISTORY_SIZE;

    pub fn associated_oracle(&self) -> Pubkey {
        self.associated_oracle
    }

    /// Extract a new price from account data
    pub fn get_price(&self, address: Pubkey, data: &[u8]) -> Result<Price> {
        if address != self.associated_oracle {
            return err!(OracleHistoryProgramError::InvalidOracleAddress);
        }
        match self.oracle_type {
            OracleType::Pyth => {
                let price_account = load_price_account(data)
                    .map_err(|_| OracleHistoryProgramError::InvalidOracleDataPyth)?;
                Ok(Price {
                    price: price_account.prev_price,
                    slot: price_account.prev_slot,
                })
            }
        }
    }

    /// Push a new price onto the history
    pub fn push(&mut self, address: Pubkey, data: &[u8]) -> Result<()> {
        let price = self.get_price(address, data)?;
        if price.slot <= self.prices.most_recent_entry().slot {
            return err!(OracleHistoryProgramError::NotNewSlot);
        }
        self.prices.push(price);
        Ok(())
    }

    /// Returns an iterator over the values, from newest to oldest.
    pub fn values(&self) -> StackVecModuloIterator<Price, ORACLE_HISTORY_SIZE> {
        StackVecModuloIterator::from(&self.prices)
    }
}