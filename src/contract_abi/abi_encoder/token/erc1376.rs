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

use crate::types::SignedRequest;

use super::ERC1376_TOKEN_INTERFACE;

pub struct Erc1376AbiEncoder;

impl Erc1376AbiEncoder {
    pub fn is_delegate_enable() -> Vec<u8> {
        let is_delegate_enable_function = &ERC1376_TOKEN_INTERFACE
            .function("isDelegateEnable")
            .expect("isDelegateEnable is always implemented; qed");

        is_delegate_enable_function
            .encode_input(&[])
            .expect("isDelegateEnable")
    }

    pub fn nonce_of(address: &Address) -> Vec<u8> {
        let nonce_of_function = &ERC1376_TOKEN_INTERFACE
            .function("nonceOf")
            .expect("nonceOf is always implemented; qed");

        nonce_of_function
            .encode_input(&[Token::Address(*address)])
            .expect("nonceOf")
    }

    pub fn delegate_transfer_and_call(req: &SignedRequest) -> Vec<u8> {
        let delegate_function = &ERC1376_TOKEN_INTERFACE
            .function("delegateTransferAndCall")
            .expect("delegateTransferAndCall is always implemented; qed");

        let req = req.unverified();
        let mut r = [0u8; 32];
        let mut s = [0u8; 32];
        req.r().to_big_endian(&mut r);
        req.s().to_big_endian(&mut s);

        let payload = delegate_function
            .encode_input(&[
                Token::Uint(*req.nonce()),
                Token::Uint(*req.fee()),
                Token::Uint(*req.gas_amount()),
                Token::Address(*req.receiver()),
                Token::Uint(*req.value()),
                Token::Bytes(req.data()),
                Token::Uint(req.delegate_mode().into()),
                Token::Uint(req.original_v().into()),
                Token::FixedBytes(r.to_vec()),
                Token::FixedBytes(s.to_vec()),
            ])
            .expect("delegateTransferAndCall");

        payload
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use ethereum_types::{Address, U256};
    use ethkey::Secret;
    use rustc_hex::ToHex;

    use crate::types::{DelegateMode, Request};

    #[test]
    fn test_delegate_transfer_and_call() {
        let sender_secret =
            Secret::from("8eeda46d11c1630bd1d9c4aace189513d3153b739f56ba6dfb5143b13dcb1eab");

        let req = Request {
            token_address: Address::from("0x89cF87c35e69A9B84F7A3e50EAf54bFc3Cabc377"),
            nonce: U256::from(77),
            fee: U256::from(6666666),
            gas_amount: U256::from(400000000),
            receiver: Address::from("0xca35b7d915458ef540ade6068dfe2f44e8fa733c"),
            value: U256::from(4898948),
            data: Vec::default(),
            delegate_mode: DelegateMode::PublicTxOrigin,
            relayer_address: Address::from("0x0000000000000000000000000000000000000000"),
        };

        let signed_req = req.sign(&sender_secret);
        let unverified_req = signed_req.unverified();
        println!(
            "v: {}, r: {}, s: {}",
            unverified_req.original_v(),
            unverified_req.r(),
            unverified_req.s(),
        );

        let data = Erc1376AbiEncoder::delegate_transfer_and_call(&signed_req);

        println!("{}", data.to_hex());
        let expected = "8b8ba692000000000000000000000000000000000000000000000000000000000000004d000000000000000000000000000000000000000000000000000000000065b9aa0000000000000000000000000000000000000000000000000000000017d78400000000000000000000000000ca35b7d915458ef540ade6068dfe2f44e8fa733c00000000000000000000000000000000000000000000000000000000004ac08400000000000000000000000000000000000000000000000000000000000001400000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000001c2cec8963ed61345e7edcbe3a57ca8139927e0eec5d0b8b82498d80d50aa2e7e07edb92f896ebdae4ae1bee735d2099c4a82ab1dabc5e6a8f0dde0ac373b186b90000000000000000000000000000000000000000000000000000000000000000";
        assert_eq!(expected, data.to_hex());
    }
}
