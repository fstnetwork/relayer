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
use std::collections::HashSet;

use ethcore::transaction::{Action, SignedTransaction, Transaction};
use ethereum_types::{Address, H256, U256};
use ethkey::Secret;

use crate::contract_abi;
use crate::types::{RequestError, SignedRequest};

use super::error::{CollationError, RequestImportError};
use super::request_converter::RequestConverter;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum CollationType {
    // empty collation
    Empty,

    // only one token transfer request
    SingleRequest,

    // multiple token transfer request
    MultipleRequest,
}

#[derive(Clone)]
pub struct RequestDispatcher<C>
where
    C: RequestConverter,
{
    // smart contract address of dispatcher
    address: Address,
    // reqeust converter
    request_converter: C,
}

#[derive(Debug, Clone)]
pub struct Collation {
    // signed token transfer request
    requests: Vec<SignedRequest>,

    // Hashes of already included requests
    request_set: HashSet<H256>,
}

impl<C> RequestDispatcher<C>
where
    C: RequestConverter,
{
    // #[inline]
    // pub fn empty() -> RequestDispatcher<C> {
    //     let request_converter = EmptyRequestConverter {};
    //     RequestDispatcher {
    //         address: Address::default(),
    //         request_converter,
    //     }
    // }

    #[inline]
    pub fn new(address: Address, request_converter: C) -> RequestDispatcher<C> {
        RequestDispatcher {
            address,
            request_converter,
        }
    }

    #[inline]
    pub fn address(&self) -> &Address {
        &self.address
    }

    #[inline]
    pub fn set_address(&mut self, address: Address) {
        self.address = address;
    }

    pub fn convert(&self, signed_requests: &Vec<SignedRequest>) -> Vec<u8> {
        self.request_converter.convert(signed_requests)
    }
}

impl Collation {
    pub fn empty() -> Collation {
        Collation {
            requests: Vec::new(),
            request_set: HashSet::new(),
        }
    }

    pub fn with_capacity(size: usize) -> Collation {
        Collation {
            requests: Vec::with_capacity(size),
            request_set: HashSet::with_capacity(size),
        }
    }

    pub fn unestimated_transaction<C: RequestConverter>(
        &self,
        request_dispatcher: &RequestDispatcher<C>,
        nonce: &U256,
        gas_price: &U256,
        value: &U256,
    ) -> Option<Transaction> {
        let (contract_address, data) = {
            match self.collation_type() {
                CollationType::Empty => {
                    return None;
                }

                CollationType::SingleRequest => {
                    let signed_req = self.requests.first().unwrap();
                    (
                        signed_req.unverified().token().clone(),
                        contract_abi::Erc1376AbiEncoder::delegate_transfer_and_call(&signed_req),
                    )
                }

                CollationType::MultipleRequest => (
                    request_dispatcher.address().clone(),
                    request_dispatcher.convert(&self.requests),
                ),
            }
        };

        Some(Transaction {
            nonce: nonce.clone(),
            gas_price: gas_price.clone(),
            gas: U256::zero(),
            action: Action::Call(contract_address),
            value: value.clone(),
            data,
        })
    }

    pub fn sign_transaction(
        &self,
        unestimated_tx: Transaction,
        gas: &U256,
        secret: &Secret,
        chain_id: Option<u64>,
    ) -> SignedTransaction {
        let mut tx = unestimated_tx;
        tx.gas = gas.clone();
        tx.sign(secret, chain_id)
    }

    fn collation_type(&self) -> CollationType {
        match self.len() {
            0 => CollationType::Empty,
            1 => CollationType::SingleRequest,
            _ => CollationType::MultipleRequest,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.requests.len()
    }
}

#[derive(Debug, Clone)]
pub struct OpenCollation {
    collation: Collation,
    unestimated_transaction: Option<Transaction>,
}

#[derive(Debug, Clone)]
pub struct ClosedCollation {
    collation: Collation,
    transaction: SignedTransaction,
}

impl OpenCollation {
    pub fn new() -> OpenCollation {
        OpenCollation {
            collation: Collation::empty(),
            unestimated_transaction: None,
        }
    }

    pub fn with_collation(collation: Collation) -> OpenCollation {
        OpenCollation {
            collation,
            unestimated_transaction: None,
        }
    }

    pub fn with_single_request(request: SignedRequest) -> OpenCollation {
        let mut collation = OpenCollation {
            collation: Collation::with_capacity(1),
            unestimated_transaction: None,
        };

        collation.push_request(request).unwrap();
        collation
    }

    pub fn with_requests(requests: Vec<SignedRequest>) -> OpenCollation {
        let mut collation = OpenCollation {
            collation: Collation::with_capacity(requests.len()),
            unestimated_transaction: None,
        };

        collation.push_requests(&requests).unwrap();
        assert_eq!(collation.collation.requests.len(), requests.len());
        assert_eq!(
            collation.collation.requests.len(),
            collation.collation.request_set.len()
        );
        collation
    }

    pub fn push_request(&mut self, request: SignedRequest) -> Result<(), RequestImportError> {
        if self.collation.request_set.contains(&request.hash()) {
            return Err(RequestError::AlreadyImported.into());
        }

        self.collation.request_set.insert(request.hash().clone());
        self.collation.requests.push(request.into());
        Ok(())
    }

    pub fn push_requests(
        &mut self,
        requests: &Vec<SignedRequest>,
    ) -> Result<(), RequestImportError> {
        requests.iter().for_each(|req| {
            self.push_request(req.clone()).unwrap();
        });

        Ok(())
    }

    pub fn update_unestimated<C: RequestConverter>(
        &mut self,
        request_dispatcher: &RequestDispatcher<C>,
        nonce: &U256,
        gas_price: &U256,
        value: &U256,
    ) {
        self.unestimated_transaction =
            self.collation
                .unestimated_transaction(request_dispatcher, nonce, gas_price, value);
    }

    pub fn unestimated(&self) -> &Option<Transaction> {
        &self.unestimated_transaction
    }

    pub fn close_with_gas(
        self,
        secret: Secret,
        chain_id: Option<u64>,
        gas: &U256,
    ) -> Result<ClosedCollation, CollationError> {
        let collation = self.collation;
        match self.unestimated_transaction {
            Some(tx) => {
                let transaction = collation.sign_transaction(tx, &gas, &secret, chain_id);
                Ok(ClosedCollation {
                    collation,
                    transaction,
                })
            }
            None => Err(CollationError::CloseWithNoTransaction),
        }
    }

    pub fn fake_close(
        &self,
        secret: Secret,
        chain_id: Option<u64>,
    ) -> Result<ClosedCollation, CollationError> {
        let collation = self.collation.clone();
        match self.unestimated_transaction.clone() {
            Some(tx) => {
                let transaction = collation.sign_transaction(tx, &U256::zero(), &secret, chain_id);
                Ok(ClosedCollation {
                    collation,
                    transaction,
                })
            }
            None => Err(CollationError::CloseWithNoTransaction),
        }
    }

    pub fn request_count(&self) -> usize {
        self.collation.requests.len()
    }

    pub fn request_hashes(&self) -> Vec<H256> {
        self.collation.request_set.iter().cloned().collect()
    }
}

impl ClosedCollation {
    pub fn is_fake(&self) -> bool {
        self.transaction.gas == U256::zero()
    }

    pub fn transaction(&self) -> &SignedTransaction {
        &self.transaction
    }

    pub fn request_count(&self) -> usize {
        self.collation.requests.len()
    }

    pub fn request_hashes(&self) -> Vec<H256> {
        self.collation.request_set.iter().cloned().collect()
    }

    pub fn reopen(self) -> OpenCollation {
        OpenCollation::with_collation(self.collation)
    }
}
