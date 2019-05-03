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
use futures::Future;
use parking_lot::{Mutex, RwLock};
use std::sync::Arc;

use crate::traits;
use crate::types::{GasEstimation, SignedRequest};

use super::{Error, InnerPool, PoolRequest, RequestSelector, VerifiedRequest};

lazy_static! {
    static ref INTRINSIC_GAS_AMOUNT: U256 = U256::from(21000);
}

fn minus_intrinsic_gas_amount(origin_gas_amount: U256) -> U256 {
    origin_gas_amount - *INTRINSIC_GAS_AMOUNT
}

pub trait Verifier {
    type Request: PoolRequest;
    type Error: Send + Sync + 'static;

    fn verify_request<S>(
        &self,
        signed_request: SignedRequest,
        insertion_id: usize,
        relayer_address: Address,
        pool: Arc<RwLock<InnerPool<Self::Request, S>>>,
    ) -> Box<Future<Item = Arc<SignedRequest>, Error = Self::Error> + Send>
    where
        S: 'static + RequestSelector<Self::Request>;
}

pub struct RequestVerifier<E>
where
    E: traits::EthereumService,
{
    ethereum: Arc<Mutex<E>>,
}

impl<E> RequestVerifier<E>
where
    E: traits::EthereumService,
{
    pub fn new(ethereum: Arc<Mutex<E>>) -> RequestVerifier<E> {
        RequestVerifier { ethereum }
    }
}

impl<E> Verifier for RequestVerifier<E>
where
    E: traits::EthereumService,
{
    type Request = VerifiedRequest;
    type Error = Error;

    fn verify_request<S>(
        &self,
        signed_request: SignedRequest,
        insertion_id: usize,
        relayer_address: Address,
        pool: Arc<RwLock<InnerPool<Self::Request, S>>>,
    ) -> Box<Future<Item = Arc<SignedRequest>, Error = Self::Error> + Send>
    where
        S: 'static + RequestSelector<Self::Request>,
    {
        Box::new(
            self.ethereum
                .lock()
                .estimate_gas(GasEstimation::TokenTransferRequest {
                    relayer_address,
                    signed_request: signed_request.clone(),
                })
            .then(move |result| match result {
                Ok(origin_gas_amount) => {
                    let calibrated_gas_amount = minus_intrinsic_gas_amount(origin_gas_amount);

                    info!(target: "pool",
                        "{}: origin estimated gas amount: {}, calibrated estimated gas amount: {}",
                        signed_request.hash(),
                        origin_gas_amount,
                        calibrated_gas_amount,
                    );
                    pool.write()
                        .import(VerifiedRequest::from_signed_with_gas_amount(
                            signed_request,
                            insertion_id,
                            calibrated_gas_amount,
                        ))
                }
                Err(err) => {
                    info!(
                        "{}: failed to estiamate gas, error: {:?}",
                        signed_request.hash(),
                        err
                    );
                    Err(
                        Error::TokenTransferRequestGasEstimationFailed,
                    )
                }
            }),
        )
    }
}
