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

use super::traits::AdminApi;

pub struct Admin<X, E>
where
    X: 'static + crate::traits::ExitHandle,
    E: 'static + crate::traits::EthereumService,
{
    exit_hanlder: Arc<Mutex<X>>,
    ethereum_service: Arc<Mutex<E>>,
}

impl<X, E> Admin<X, E>
where
    X: 'static + crate::traits::ExitHandle,
    E: 'static + crate::traits::EthereumService,
{
    pub fn new(exit_hanlder: Arc<Mutex<X>>, ethereum_service: Arc<Mutex<E>>) -> Admin<X, E> {
        Admin {
            exit_hanlder,
            ethereum_service,
        }
    }
}

impl<X, E> AdminApi for Admin<X, E>
where
    X: 'static + crate::traits::ExitHandle,
    E: 'static + crate::traits::EthereumService,
{
    fn shutdown(&self) -> Result<bool> {
        self.exit_hanlder.lock().shutdown();
        Ok(true)
    }

    fn add_ethereum_endpoint(&self, endpoint: String) -> Result<bool> {
        Ok(self.ethereum_service.lock().add_endpoint(endpoint))
    }

    fn remove_ethereum_endpoint(&self, endpoint: String) -> Result<bool> {
        Ok(self.ethereum_service.lock().remove_endpoint(&endpoint))
    }

    fn ethereum_endpoints(&self) -> Result<Vec<String>> {
        Ok(self.ethereum_service.lock().endpoints())
    }

    fn ethereum_endpoint_count(&self) -> Result<usize> {
        Ok(self.ethereum_service.lock().endpoint_count())
    }
}
