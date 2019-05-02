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
use crate::types::SignedRequest;

// RequestConverter converts a set of signed token transfer request to "data" of a transaction
pub trait RequestConverter: Clone + Send + Sync {
    fn convert(&self, _requests: &Vec<SignedRequest>) -> Vec<u8> {
        Vec::default()
    }
}

#[derive(Clone)]
pub struct EmptyRequestConverter {}
#[derive(Clone)]
pub struct FstRequestConverter {}

impl RequestConverter for EmptyRequestConverter {}

impl FstRequestConverter {
    pub fn new() -> FstRequestConverter {
        FstRequestConverter {}
    }
}

impl RequestConverter for FstRequestConverter {
    fn convert(&self, requests: &Vec<SignedRequest>) -> Vec<u8> {
        use crate::contract_abi::FstTokenTransferRequestDispatcherAbiEncoder;

        let encode = match is_single_token(requests) {
            true => FstTokenTransferRequestDispatcherAbiEncoder::single_token_dispatch,
            false => FstTokenTransferRequestDispatcherAbiEncoder::multiple_token_dispatch,
        };
        encode(requests)
    }
}

fn is_single_token(requests: &Vec<SignedRequest>) -> bool {
    let token_address = match requests.first() {
        Some(first) => first.unverified().token(),
        None => return false,
    };
    requests
        .iter()
        .all(|req| req.unverified().token() == token_address)
}
