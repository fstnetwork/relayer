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
use parking_lot::Mutex;
use std::sync::Arc;

use jsonrpc_core::Result;

use super::traits::PoolApi;

pub struct Pool<P> {
    pool: Arc<Mutex<P>>,
}

impl<P> Pool<P>
where
    P: 'static + traits::PoolService<SignedRequest = types::SignedRequest>,
{
    pub fn new(pool: Arc<Mutex<P>>) -> Pool<P> {
        Pool { pool }
    }
}

impl<P> PoolApi for Pool<P>
where
    P: 'static + traits::PoolService<SignedRequest = types::SignedRequest>,
{
    fn version(&self) -> Result<String> {
        Ok("0.1.0".to_owned())
    }

    fn all_requests(&self) -> Result<Vec<types::RelayerRpcRequest>> {
        let pool = self.pool.lock();
        let requests = pool
            .all_requests()
            .iter()
            .map(|req| types::RelayerRpcRequest::from(req.as_ref()))
            .collect();
        Ok(requests)
    }

    fn request_count(&self) -> Result<usize> {
        let pool = self.pool.lock();
        let count = pool.all_requests().len();
        Ok(count)
    }

    fn ready_requests(&self) -> Result<Vec<types::RelayerRpcRequest>> {
        let requests: Vec<_> = self
            .pool
            .lock()
            .ready_requests(
                None,
                traits::PoolPendingSettings {
                    filter: None,
                    gas_limit: 0.into(),
                    relayer: None,
                },
            )
            .iter()
            .map(|req| types::RelayerRpcRequest::from(req.as_ref()))
            .collect();

        Ok(requests)
    }

    fn future_requests(&self) -> Result<Vec<types::RelayerRpcRequest>> {
        let reqs = vec![];
        Ok(reqs)
    }
}
