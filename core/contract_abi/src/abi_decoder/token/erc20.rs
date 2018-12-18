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
use ethereum_types::U256;

use super::error::{Error, ErrorKind};
use super::ERC20_TOKEN_INTERFACE;

pub struct ERC20AbiDecoder;

impl ERC20AbiDecoder {
    pub fn symbol(data: &Vec<u8>) -> Result<String, Error> {
        let symbol_function = &ERC20_TOKEN_INTERFACE
            .function("symbol")
            .expect("symbol is always implemented; qed");

        let mut vec = symbol_function.decode_output(data)?;
        match vec.pop() {
            Some(Token::String(symbol)) => Ok(symbol),
            _ => Err(Error::from(ErrorKind::InvalidReturnValue)),
        }
    }

    pub fn balance_of(data: &Vec<u8>) -> Result<U256, Error> {
        let balance_of_function = &ERC20_TOKEN_INTERFACE
            .function("balanceOf")
            .expect("balanceOf is always implemented; qed");

        let mut vec = balance_of_function.decode_output(data)?;
        match vec.pop() {
            Some(Token::Uint(balance)) => Ok(balance),
            _ => Err(Error::from(ErrorKind::InvalidReturnValue)),
        }
    }

    pub fn total_supply(data: &Vec<u8>) -> Result<U256, Error> {
        let total_supply_function = &ERC20_TOKEN_INTERFACE
            .function("totalSupply")
            .expect("totalSupply is always implemented; qed");

        let mut vec = total_supply_function.decode_output(data)?;
        match vec.pop() {
            Some(Token::Uint(total_supply)) => Ok(total_supply),
            _ => Err(Error::from(ErrorKind::InvalidReturnValue)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol() {
        let expected = "SUMMER".to_owned();
        let data = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x53, 0x55, 0x4d, 0x4d, 0x45, 0x52,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        assert_eq!(expected, ERC20AbiDecoder::symbol(&data).unwrap());
    }

    #[test]
    fn test_balance_of() {
        let expected = U256::from_dec_str("1228999983998999617860").unwrap();
        let data = vec![
            00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00,
            0x00, 0x42, 0x9f, 0xcd, 0x0d, 0xed, 0x15, 0x22, 0x61, 0x44,
        ];
        assert_eq!(expected, ERC20AbiDecoder::balance_of(&data).unwrap());
    }
}
