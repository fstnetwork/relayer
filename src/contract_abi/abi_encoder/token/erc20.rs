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
use ethabi::Token;
use ethereum_types::Address;

use super::ERC20_TOKEN_INTERFACE;

pub struct Erc20AbiEncoder;

impl Erc20AbiEncoder {
    pub fn total_supply() -> Vec<u8> {
        let total_supply_function = &ERC20_TOKEN_INTERFACE
            .function("totalSupply")
            .expect("totalSupply is always implemented; qed");
        total_supply_function
            .encode_input(&[])
            .expect("totalSupply")
    }

    pub fn symbol() -> Vec<u8> {
        let symbol_function = &ERC20_TOKEN_INTERFACE
            .function("symbol")
            .expect("symbol is always implemented; qed");
        symbol_function.encode_input(&[]).expect("symbol")
    }

    pub fn balance_of(address: &Address) -> Vec<u8> {
        let balance_of_function = &ERC20_TOKEN_INTERFACE
            .function("balanceOf")
            .expect("balanceOf is always implemented; qed");

        balance_of_function
            .encode_input(&[Token::Address(*address)])
            .expect("balanceOf")
    }
}
