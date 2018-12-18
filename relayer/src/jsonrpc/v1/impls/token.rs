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
use ethereum_types::{Address, H256, U256};
use futures::Future;
use jsonrpc_core::{BoxFuture, Error, ErrorCode, Result};
use parking_lot::Mutex;
use std::collections::HashSet;
use std::sync::Arc;

use super::traits::TokenApi;

pub struct Token<E, P>
where
    E: 'static + traits::EthereumService,
    P: 'static + traits::PoolService<SignedRequest = types::SignedRequest>,
{
    ethereum: Arc<Mutex<E>>,
    pool: Arc<Mutex<P>>,
}

impl<E, P> Token<E, P>
where
    E: 'static + traits::EthereumService,
    P: 'static + traits::PoolService<SignedRequest = types::SignedRequest>,
{
    pub fn new(ethereum: Arc<Mutex<E>>, pool: Arc<Mutex<P>>) -> Token<E, P> {
        Token { ethereum, pool }
    }
}

impl<E, P> TokenApi for Token<E, P>
where
    E: 'static + traits::EthereumService,
    P: 'static + traits::PoolService<SignedRequest = types::SignedRequest>,
{
    fn is_delegate_enable(&self, token_contract: Address) -> BoxFuture<bool> {
        Box::new(
            self.ethereum
                .lock()
                .token_delegate_enable(token_contract)
                .map_err(|_| Error {
                    code: ErrorCode::InvalidParams,
                    message: "invalid token delegate enable request".to_owned(),
                    data: None,
                }),
        )
    }

    fn balance_of(&self, token: Address, account: Address) -> BoxFuture<U256> {
        Box::new(
            self.ethereum
                .lock()
                .balance_of(account, types::Currency::Token(token))
                .map_err(|_| Error {
                    code: ErrorCode::InvalidParams,
                    message: "invalid balance request".to_owned(),
                    data: None,
                }),
        )
    }

    fn nonce_of(&self, token: Address, account: Address) -> BoxFuture<U256> {
        Box::new(
            self.ethereum
                .lock()
                .nonce_of(account, types::Currency::Token(token))
                .map_err(|_| Error {
                    code: ErrorCode::InvalidParams,
                    message: "invalid nonce request".to_owned(),
                    data: None,
                }),
        )
    }

    fn send_token_transfer_request(&self, request: types::RelayerRpcRequest) -> BoxFuture<H256> {
        let signed_req = match request.into_signed_request() {
            Ok(signed_req) => signed_req,
            Err(err) => {
                warn!("invalid token transfer request {:?}", err);
                return Box::new(futures::future::err(Error {
                    code: ErrorCode::InvalidParams,
                    message: "failed to deserialize token transfer request".to_owned(),
                    data: None,
                }));
            }
        };

        info!(
            "received new token transafer request: {:?}",
            signed_req.hash()
        );

        Box::new(
            self.pool
                .lock()
                .import(signed_req)
                .then(|result| match result {
                    Ok(signed_req) => {
                        let hash = signed_req.hash();
                        info!("import new token transafer request: {:?}", hash);
                        Ok(*hash)
                    }
                    Err(err) => Err(Error {
                        code: ErrorCode::InvalidParams,
                        message: err.to_string(),
                        data: None,
                    }),
                }),
        )
    }

    fn send_token_transfer_requests(
        &self,
        requests: Vec<types::RelayerRpcRequest>,
    ) -> BoxFuture<Vec<H256>> {
        let signed_requests = requests
            .into_iter()
            .fold(HashSet::new(), |mut reqs, request| {
                request
                    .into_signed_request()
                    .map(|signed_req| {
                        info!(
                            "received new token transafer request: {:?}",
                            signed_req.hash()
                        );
                        reqs.insert(signed_req);
                    })
                    .map_err(|_| {})
                    .expect("; qed");
                reqs
            });

        // FIXME
        Box::new(futures::future::ok(
            signed_requests.into_iter().map(|req| *req.hash()).collect(),
        ))

        // let futures: Vec<_> = signed_requests
        //     .into_iter()
        //     .map(|req| self.pool.lock().import(req))
        //     .collect();
        // Box::new(futures::future::ok(Vec::new()))
        //
        // Box::new(futures::future::join_all(futures).and_then(|reqs| {
        //     let hashes: Vec<_> = reqs
        //         .iter()
        //         .map(|req| {
        //             let hash = req.hash();
        //             info!("import new token transafer request: {:?}", hash);
        //             *hash
        //         }).collect();
        //     hashes
        // }))
    }

    fn supported_tokens(&self) -> Result<Vec<types::RelayerRpcToken>> {
        let tokens = vec![];
        Ok(tokens)
    }
}
