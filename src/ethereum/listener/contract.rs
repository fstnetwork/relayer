use ethereum_types::{Address, U256};
use std::collections::HashMap;

use crate::types::AccountState;

trait Erc20Like {
    fn address(&self) -> &Address;
    fn symbol(&self) -> &str;
    fn balance(&self, address: &Address) -> U256;
    fn set_balance(&mut self, address: Address, balance: U256);
    fn balances(&self) -> &HashMap<Address, U256>;
    fn set_balances(&mut self, balances: HashMap<Address, U256>);
    fn account_state(&self, address: Address) -> AccountState;
}

pub struct Erc1376Contract {
    address: Address,
    symbol: String,
    balances: HashMap<Address, U256>,
    nonces: HashMap<Address, U256>,
}

impl Erc1376Contract {
    pub fn new(address: Address, symbol: String) -> Erc1376Contract {
        Erc1376Contract {
            address,
            symbol,
            balances: Default::default(),
            nonces: Default::default(),
        }
    }

    pub fn with_balances_and_nonces(
        address: Address,
        symbol: String,
        balances: HashMap<Address, U256>,
        nonces: HashMap<Address, U256>,
    ) -> Erc1376Contract {
        Erc1376Contract {
            address,
            symbol,
            balances,
            nonces,
        }
    }

    pub fn nonce(&self, address: &Address) -> U256 {
        let zero = U256::zero();
        *self.nonces.get(address).unwrap_or(&zero)
    }

    pub fn set_nonce(&mut self, address: Address, nonce: U256) {
        self.nonces.insert(address, nonce);
    }

    pub fn nonces(&self) -> &HashMap<Address, U256> {
        &self.nonces
    }

    pub fn set_nonces(&mut self, nonces: HashMap<Address, U256>) {
        self.nonces = nonces
    }
}

impl Erc20Like for Erc1376Contract {
    fn address(&self) -> &Address {
        &self.address
    }

    fn symbol(&self) -> &str {
        &self.symbol
    }

    fn account_state(&self, address: Address) -> AccountState {
        let zero = U256::zero();
        let nonce = self.nonces.get(&address).unwrap_or(&zero);
        let balance = self.balances.get(&address).unwrap_or(&zero);
        AccountState::new(address, nonce.clone(), balance.clone())
    }

    fn balance(&self, address: &Address) -> U256 {
        let zero = U256::zero();
        *self.balances.get(address).unwrap_or(&zero)
    }

    fn set_balance(&mut self, address: Address, balance: U256) {
        self.balances.insert(address, balance);
    }

    fn balances(&self) -> &HashMap<Address, U256> {
        &self.balances
    }

    fn set_balances(&mut self, balances: HashMap<Address, U256>) {
        self.balances = balances;
    }
}
