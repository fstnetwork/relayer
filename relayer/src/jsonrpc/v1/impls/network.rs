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
use jsonrpc_core::Result;
use parking_lot::Mutex;
use std::sync::Arc;

use super::traits::network::NetworkApi;

pub struct Network<N> {
    network: Arc<Mutex<N>>,
}

impl<N> Network<N>
where
    N: 'static + traits::NetworkService,
{
    pub fn new(network: Arc<Mutex<N>>) -> Self {
        Network { network }
    }
}

impl<N> NetworkApi for Network<N>
where
    N: 'static + traits::NetworkService,
{
    fn version(&self) -> Result<String> {
        Ok("0.1.0".to_owned())
    }

    fn peer_count(&self) -> Result<u32> {
        Ok(self.network.lock().peer_count() as u32)
    }

    fn is_listening(&self) -> Result<bool> {
        Ok(self.network.lock().is_listening())
    }
}
