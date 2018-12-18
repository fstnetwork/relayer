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

use types::SignedRequest;

use super::ERC1376AbiEncoder;
use super::FST_TOKEN_TRANSFER_REQUEST_DISPATCHER_INTERFACE;

pub struct FstTokenTransferRequestDispatcherAbiEncoder;

impl FstTokenTransferRequestDispatcherAbiEncoder {
    fn encode_payloads(requests: &Vec<SignedRequest>) -> Vec<Token> {
        requests
            .iter()
            .map(|req| Token::Bytes(ERC1376AbiEncoder::delegate_transfer_and_call(req)))
            .collect()
    }

    pub fn single_token_dispatch(requests: &Vec<SignedRequest>) -> Vec<u8> {
        assert!(!requests.is_empty());

        let first_req = requests.first().unwrap();
        let token_address = first_req.unverified().token();
        assert!(requests
            .iter()
            .all(move |req| req.unverified().token() == token_address));

        let dispatch_function = FST_TOKEN_TRANSFER_REQUEST_DISPATCHER_INTERFACE
            .function("singleTokenDispatch")
            .unwrap();
        let payloads = Self::encode_payloads(requests);

        dispatch_function
            .encode_input(&[Token::Address(*token_address), Token::Array(payloads)])
            .unwrap()
    }

    pub fn multiple_token_dispatch(requests: &Vec<SignedRequest>) -> Vec<u8> {
        let dispatch_function = FST_TOKEN_TRANSFER_REQUEST_DISPATCHER_INTERFACE
            .function("multipleTokenDispatch")
            .unwrap();

        let token_addresses = requests
            .iter()
            .map(|req| Token::Address(*req.unverified().token()))
            .collect();
        let payloads = Self::encode_payloads(requests);

        dispatch_function
            .encode_input(&[Token::Array(token_addresses), Token::Array(payloads)])
            .unwrap()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use ethereum_types::{Address, U256};
    use ethkey::Secret;

    use rustc_hex::ToHex;
    use types::{DelegateMode, Request, SignedRequest};

    #[test]
    #[should_panic]
    fn test_empty_request() {
        FstTokenTransferRequestDispatcherAbiEncoder::single_token_dispatch(&vec![]);
    }

    #[test]
    fn test_single_token_dispatch() {
        let expected = "aa9f1410000000000000000000000000cab77b4b9bf9b92a53572091c5798c570051be8f00000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000001648b8ba692000000000000000000000000000000000000000000000000000000000000004d000000000000000000000000000000000000000000000000000000000065b9aa0000000000000000000000000000000000000000000000000000000017d78400000000000000000000000000ca35b7d915458ef540ade6068dfe2f44e8fa733c00000000000000000000000000000000000000000000000000000000004ac08400000000000000000000000000000000000000000000000000000000000001400000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000001b8357d3f1c70d186fd1ef8ec18672d53a25e005216d19e6cb3aab0f60844a562e00d7e825ad6146fea4619d7d971d00683d3fdd432a203af9450f47158cc7c2e400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001648b8ba692000000000000000000000000000000000000000000000000000000000000004d000000000000000000000000000000000000000000000000000000000065b9aa0000000000000000000000000000000000000000000000000000000017d78400000000000000000000000000ca35b7d915458ef540ade6068dfe2f44e8fa733c00000000000000000000000000000000000000000000000000000000004ac08400000000000000000000000000000000000000000000000000000000000001400000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000001b8357d3f1c70d186fd1ef8ec18672d53a25e005216d19e6cb3aab0f60844a562e00d7e825ad6146fea4619d7d971d00683d3fdd432a203af9450f47158cc7c2e4000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        // let expected = "aa9f1410000000000000000000000000cab77b4b9bf9b92a53572091c5798c570051be8f0000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000024000000000000000000000000000000000000000000000000000000000000001648b8ba692000000000000000000000000000000000000000000000000000000000000004d000000000000000000000000000000000000000000000000000000000065b9aa0000000000000000000000000000000000000000000000000000000017d78400000000000000000000000000ca35b7d915458ef540ade6068dfe2f44e8fa733c00000000000000000000000000000000000000000000000000000000004ac08400000000000000000000000000000000000000000000000000000000000001400000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000001b8357d3f1c70d186fd1ef8ec18672d53a25e005216d19e6cb3aab0f60844a562e00d7e825ad6146fea4619d7d971d00683d3fdd432a203af9450f47158cc7c2e400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001648b8ba692000000000000000000000000000000000000000000000000000000000000004d000000000000000000000000000000000000000000000000000000000065b9aa0000000000000000000000000000000000000000000000000000000017d78400000000000000000000000000ca35b7d915458ef540ade6068dfe2f44e8fa733c00000000000000000000000000000000000000000000000000000000004ac08400000000000000000000000000000000000000000000000000000000000001400000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000001b8357d3f1c70d186fd1ef8ec18672d53a25e005216d19e6cb3aab0f60844a562e00d7e825ad6146fea4619d7d971d00683d3fdd432a203af9450f47158cc7c2e4000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";

        let sender_secret =
            Secret::from("8eeda46d11c1630bd1d9c4aace189513d3153b739f56ba6dfb5143b13dcb1eab");

        let req = Request {
            token_address: Address::from("0xcab77b4b9bf9b92a53572091c5798c570051be8f"),
            nonce: U256::from(77),
            fee: U256::from(6666666),
            gas_amount: U256::from(400000000),
            receiver: Address::from("0xca35b7d915458ef540ade6068dfe2f44e8fa733c"),
            value: U256::from(4898948),
            data: Vec::default(),
            delegate_mode: DelegateMode::PublicTxOrigin,
            relayer_address: Address::from("0x0000000000000000000000000000000000000000"),
        };

        let signed_req1 = req.clone().sign(&sender_secret);
        let signed_req2 = req.clone().sign(&sender_secret);
        let data = &FstTokenTransferRequestDispatcherAbiEncoder::single_token_dispatch(&vec![
            signed_req1,
            signed_req2,
        ]);

        assert_eq!(expected, data.to_hex());
    }

    #[test]
    fn test_multiple_token_dispatch() {
        let sender_secret =
            Secret::from("8eeda46d11c1630bd1d9c4aace189513d3153b739f56ba6dfb5143b13dcb1eab");

        let requests: Vec<SignedRequest> = [0u32; 10]
            .into_iter()
            .map(|index| {
                Request {
                    token_address: Address::from("0x89cf87c35e69a9b84f7a3E50eaF54bfc3cabc377"),
                    nonce: U256::from(0x81 + index),
                    fee: U256::from("deadbeef"),
                    gas_amount: U256::from(0xdf2c38),
                    receiver: Address::from("0x7195eb47570cF0aeCe30893e8e7e56C4Da5f0AC2"),
                    value: U256::from("deadbeef5524"),
                    data: Vec::default(),
                    delegate_mode: DelegateMode::PublicMsgSender,
                    relayer_address: Address::from("0x0000000000000000000000000000000000000000"),
                }
                .sign(&sender_secret)
            })
            .collect();

        let data = &FstTokenTransferRequestDispatcherAbiEncoder::multiple_token_dispatch(&requests);
        println!("{}", data.to_hex(),);
    }
}
