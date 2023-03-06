use crate::errors::OracleHistoryProgramError;
use crate::svec_modulo::{StackVecModulo, StackVecModuloIterator};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Slot;
use pyth_sdk_solana::state::load_price_account;

/// Sets the size of oracle price history accounts.
///
/// Cannot be larger than `(10240 - header) / mem::sizeof::<Price>()`.
const ORACLE_HISTORY_SIZE: usize = 100;

/// A record of a price at a given time..
/// The purpose of oracle history accounts is to store this data.
///
/// If you wanted to target some other kind of oracle data,
/// you'd do so by replacing this struct with some other data type.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[repr(C)]
pub struct Price {
    pub price: i64,
    pub slot: Slot,
}

/// Determines how to deserialize the associated oracle account
/// when indexing.
#[derive(Copy, Clone, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
#[repr(C)]
pub enum OracleType {
    #[default]
    Pyth,
}

/// Data account, stores oracle price data and a header that describes
/// the target oracle and a data type enum which tells how to deserialize the data.
#[account(zero_copy)]
#[derive(Default, Debug)]
#[repr(C)]
pub struct OracleHistory {
    pub(crate) associated_oracle: Pubkey,
    pub(crate) oracle_type: OracleType,
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

    pub fn most_recent_entry(&self) -> &Price {
        self.prices.most_recent_entry()
    }

    pub fn most_recent_index(&self) -> usize {
        self.prices.most_recent_index()
    }

    pub fn len(&self) -> usize {
        self.prices.len()
    }

    /// Returns an iterator over the values, from newest to oldest.
    pub fn values(&self) -> StackVecModuloIterator<Price, ORACLE_HISTORY_SIZE> {
        StackVecModuloIterator::from(&self.prices)
    }
}

#[cfg(test)]
mod tests {
    use anchor_lang::prelude::Pubkey;
    use anchor_lang::solana_program::pubkey;
    use base64::Engine;
    use crate::state::OracleHistory;

    const ORACLE_DATA: &str = include_str!("../../../tests/oracle_data");
    const ORACLE_ACCOUNT: Pubkey = pubkey!("H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG");

    #[test]
    fn get_price() {
        let data = base64::engine::general_purpose::STANDARD.decode(ORACLE_DATA.trim_end()).unwrap();
        let mut state = OracleHistory::default();
        state.associated_oracle = ORACLE_ACCOUNT;
        let price = state.get_price(
            ORACLE_ACCOUNT,
            &data
        ).unwrap();
        state.push(
            ORACLE_ACCOUNT,
            &data
        ).unwrap();
        assert_eq!(*state.most_recent_entry(), price);
    }

    #[test]
    #[should_panic]
    fn get_price_bad_address() {
        let data = base64::engine::general_purpose::STANDARD.decode(ORACLE_DATA.trim_end()).unwrap();
        let mut state = OracleHistory::default();
        state.associated_oracle = ORACLE_ACCOUNT;
        let _ = state.get_price(
            Pubkey::default(),
            &data
        ).unwrap();
    }

    #[test]
    #[should_panic]
    fn get_price_not_new_slot() {
        let data = base64::engine::general_purpose::STANDARD.decode(ORACLE_DATA.trim_end()).unwrap();
        let mut state = OracleHistory::default();
        state.associated_oracle = ORACLE_ACCOUNT;
        state.push(
            ORACLE_ACCOUNT,
            &data
        ).unwrap();
        state.push(
            ORACLE_ACCOUNT,
            &data
        ).unwrap();
    }
}
