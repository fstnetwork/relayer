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

#[derive(Debug, Clone)]
pub struct AccountState {
    address: Address,
    nonce: U256,
    balance: U256,
}

impl Default for AccountState {
    fn default() -> AccountState {
        Self::empty()
    }
}

impl AccountState {
    pub fn new(address: Address, nonce: U256, balance: U256) -> AccountState {
        AccountState {
            address,
            nonce,
            balance,
        }
    }

    pub fn empty() -> AccountState {
        Self::new(Address::zero(), U256::zero(), U256::zero())
    }

    pub fn with_address(address: Address) -> AccountState {
        Self::new(address.clone(), U256::zero(), U256::zero())
    }

    pub fn address(&self) -> &Address {
        &self.address
    }

    pub fn nonce(&self) -> &U256 {
        &self.nonce
    }

    pub fn balance(&self) -> &U256 {
        &self.balance
    }

    pub fn update_balance(&mut self, balance: U256) {
        self.balance = balance;
    }

    pub fn update_nonce(&mut self, nonce: U256) {
        self.nonce = nonce;
    }

    pub fn increase_nonce(&mut self) {
        self.nonce += U256::one();
    }

    pub fn fetch_and_increase_nonce(&mut self) -> U256 {
        let nonce = self.nonce;
        self.nonce += U256::one();
        nonce
    }
}
