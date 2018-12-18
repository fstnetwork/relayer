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
use super::ERC1376_TOKEN_INTERFACE;

pub struct ERC1376AbiDecoder;

impl ERC1376AbiDecoder {
    pub fn is_delegate_enable(data: &Vec<u8>) -> Result<bool, Error> {
        let function = &ERC1376_TOKEN_INTERFACE
            .function("isDelegateEnable")
            .expect("isDelegateEnable is always implemented; qed");

        let mut vec = function.decode_output(data)?;
        match vec.pop() {
            Some(Token::Bool(enable)) => Ok(enable),
            _ => Err(Error::from(ErrorKind::InvalidReturnValue)),
        }
    }

    pub fn nonce_of(data: &Vec<u8>) -> Result<U256, Error> {
        let nonce_of_function = &ERC1376_TOKEN_INTERFACE
            .function("nonceOf")
            .expect("nonceOf is always implemented; qed");

        let mut vec = nonce_of_function.decode_output(data)?;
        match vec.pop() {
            Some(Token::Uint(nonce)) => Ok(nonce),
            _ => Err(Error::from(ErrorKind::InvalidReturnValue)),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_nonce_of() {
        let expected = U256::from(0x2224);
        let data = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x22, 0x24,
        ];

        assert_eq!(expected, ERC1376AbiDecoder::nonce_of(&data).unwrap());
    }
}
