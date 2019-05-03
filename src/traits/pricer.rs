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
use ethereum_types::{Address, U256};
use futures::Future;

pub trait PriceService: Future + Sync + Send {
    type PricerError: Sync + Send + 'static;

    fn gas_price(&self) -> Box<Future<Item = U256, Error = Self::PricerError> + Send> {
        Box::new(futures::future::ok(U256::zero()))
    }

    fn token_price(
        &self,
        _token: Address,
    ) -> Box<Future<Item = U256, Error = Self::PricerError> + Send> {
        Box::new(futures::future::ok(U256::zero()))
    }
}
