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
#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

extern crate ethereum_types;
extern crate futures;
extern crate parking_lot;
extern crate rustc_hex;
extern crate smallvec;
extern crate tokio_timer;

extern crate traits;
extern crate types;

use ethereum_types::{Address, H256, U256};
use std::sync::Arc;
use std::{cmp, fmt};

use traits::PoolRequestTag;
use types::{DelegateMode, SignedRequest, UnverifiedRequest};

mod error;
mod filter;
mod inner;
mod params;
mod queue;
mod ready;
mod selector;
mod service;
mod status;
mod verifier;

#[cfg(test)]
mod tests;

pub use self::service::Service as PoolService;

pub use self::error::{Error, ErrorKind};
pub use self::filter::{
    AddressFilter, DummyAddressFilter, ListAddressFilter, ListAddressFilterMode,
};
pub use self::params::Params as PoolParams;
pub use self::ready::{Readiness, ReadyChecker};
pub use self::selector::{NonceAndFeeSelector, RequestSelector, TokenSelector};
pub use self::status::Status;
pub use self::verifier::{RequestVerifier, Verifier};

use self::inner::InnerPool;
use self::queue::{AddResult, RequestQueue};

pub trait PoolRequest: fmt::Debug + Clone {
    fn clone_signed(&self) -> Arc<SignedRequest>;

    fn insertion_id(&self) -> usize;

    fn signed(&self) -> &SignedRequest;

    fn hash(&self) -> &H256;

    fn token(&self) -> &Address;

    fn sender(&self) -> &Address;

    fn relayer(&self) -> &Address;

    fn nonce(&self) -> &U256;

    fn fee(&self) -> &U256;

    fn gas_amount(&self) -> &U256;

    fn update_gas_amount(&mut self, gas_amount: U256);

    fn delegate_mode(&self) -> DelegateMode;
}

#[derive(Debug, Clone)]
pub struct VerifiedRequest {
    inner: Arc<SignedRequest>,
    insertion_id: usize,
    gas_amount: U256,
}

impl VerifiedRequest {
    pub fn from_signed(signed: SignedRequest, insertion_id: usize) -> VerifiedRequest {
        VerifiedRequest {
            inner: Arc::new(signed),
            insertion_id,
            gas_amount: 0.into(),
        }
    }

    pub fn from_signed_with_gas_amount(
        signed: SignedRequest,
        insertion_id: usize,
        gas_amount: U256,
    ) -> VerifiedRequest {
        VerifiedRequest {
            inner: Arc::new(signed),
            insertion_id,
            gas_amount,
        }
    }

    fn unverified(&self) -> &UnverifiedRequest {
        self.inner.unverified()
    }
}

impl PoolRequest for VerifiedRequest {
    #[inline]
    fn insertion_id(&self) -> usize {
        self.insertion_id
    }

    #[inline]
    fn signed(&self) -> &SignedRequest {
        &self.inner
    }

    #[inline]
    fn clone_signed(&self) -> Arc<SignedRequest> {
        self.inner.clone()
    }

    #[inline]
    fn hash(&self) -> &H256 {
        self.inner.hash()
    }

    #[inline]
    fn sender(&self) -> &Address {
        self.inner.sender()
    }

    #[inline]
    fn nonce(&self) -> &U256 {
        self.unverified().nonce()
    }

    #[inline]
    fn fee(&self) -> &U256 {
        self.unverified().fee()
    }

    #[inline]
    fn token(&self) -> &Address {
        self.unverified().token()
    }

    #[inline]
    fn relayer(&self) -> &Address {
        self.unverified().relayer_address()
    }

    #[inline]

    fn delegate_mode(&self) -> DelegateMode {
        self.unverified().delegate_mode()
    }

    #[inline]
    fn gas_amount(&self) -> &U256 {
        &self.gas_amount
    }

    #[inline]
    fn update_gas_amount(&mut self, gas_amount: U256) {
        self.gas_amount = gas_amount;
    }
}

#[derive(Debug)]
pub struct ScoredRequest<S: Send, R: PoolRequest> {
    score: S,
    request: R,
}

impl<S: Send, R: PoolRequest> ScoredRequest<S, R> {
    pub fn new(score: S, request: R) -> ScoredRequest<S, R> {
        ScoredRequest { score, request }
    }

    pub fn score(&self) -> &S {
        &self.score
    }

    pub fn request(&self) -> &R {
        &self.request
    }
}

impl<S: Clone + Send, R: PoolRequest> Clone for ScoredRequest<S, R> {
    fn clone(&self) -> ScoredRequest<S, R> {
        ScoredRequest {
            request: self.request.clone(),
            score: self.score.clone(),
        }
    }
}

impl<S: cmp::Ord + Send, R: PoolRequest> Ord for ScoredRequest<S, R> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        other.score.cmp(&self.score).then(
            self.request
                .insertion_id()
                .cmp(&other.request.insertion_id()),
        )
    }
}

impl<S: cmp::Ord + Send, R: PoolRequest> PartialOrd for ScoredRequest<S, R> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<S: cmp::Ord + Send, R: PoolRequest> PartialEq for ScoredRequest<S, R> {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score && self.request.insertion_id() == other.request.insertion_id()
    }
}

impl<R: PoolRequest, S: cmp::Ord + Send> Eq for ScoredRequest<S, R> {}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Choice {
    InsertNew,
    RejectNew,
    ReplaceOld,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change<R = ()> {
    InsertedAt(usize),
    RemovedAt(usize),
    ReplacedAt(usize),
    Culled(usize),
    Event(R),
}
