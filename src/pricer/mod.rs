// Copyright 2017-2018 FST Network Pte. Ltd.
// This file is part of FST Relayer.

// FST Relayer is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// FST Relayer is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with FST Relayer. If not, see <http://www.gnu.org/licenses/>.
mod error;

use ethereum_types::{Address, U256};
use futures::{Async, Future, Poll, Stream};
use std::collections::HashMap;

use crate::traits;

pub use self::error::Error;

lazy_static! {
    static ref WEI: U256 = U256::from(1);
    static ref KWEI: U256 = *WEI * U256::from(1000);
    static ref MWEI: U256 = *KWEI * U256::from(1000);
    static ref GWEI: U256 = *MWEI * U256::from(1000);
    static ref ETHER: U256 = U256::from(10).pow(U256::from(18));
    static ref FIXED_GAS_PRICE: U256 = U256::from(1) * (*GWEI);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PriceServiceMode {
    Fixed {
        gas_price: U256,
        token_prices: HashMap<Address, U256>,
    },
    Calibrate,
}

pub struct PriceService {
    mode: PriceServiceMode,
    gas_price: U256,
    token_prices: HashMap<Address, U256>,
}

impl PriceService {
    pub fn new(mode: PriceServiceMode) -> PriceService {
        match mode.clone() {
            PriceServiceMode::Fixed {
                gas_price,
                token_prices,
            } => PriceService {
                mode,
                gas_price,
                token_prices,
            },
            PriceServiceMode::Calibrate => PriceService {
                mode,
                gas_price: *FIXED_GAS_PRICE,
                token_prices: HashMap::new(),
            },
        }
    }
}

impl Default for PriceService {
    fn default() -> PriceService {
        PriceService {
            mode: PriceServiceMode::Fixed {
                gas_price: *FIXED_GAS_PRICE,
                token_prices: {
                    let prices = HashMap::new();
                    prices
                },
            },
            gas_price: *FIXED_GAS_PRICE,
            token_prices: HashMap::new(),
        }
    }
}

impl Drop for PriceService {
    fn drop(&mut self) {}
}

impl traits::PriceService for PriceService {
    type PricerError = Error;

    fn gas_price(&self) -> Box<Future<Item = U256, Error = Self::PricerError> + Send> {
        Box::new(futures::future::ok(self.gas_price))
    }

    fn token_price(
        &self,
        token: Address,
    ) -> Box<Future<Item = U256, Error = Self::PricerError> + Send> {
        Box::new(futures::future::ok(
            self.token_prices
                .get(&token)
                .unwrap_or(&U256::from(100))
                .clone(),
        ))
    }
}

impl Stream for PriceService {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            return Ok(Async::NotReady);
        }
    }
}
