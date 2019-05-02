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

build_rpc_trait! {
    pub trait AdminApi {

        #[rpc(name="admin_shutdown")]
        fn shutdown(&self) -> Result<bool>;

        #[rpc(name="admin_addEthereumEndpoint")]
        fn add_ethereum_endpoint(&self, String) -> Result<bool>;

        #[rpc(name="admin_removeEthereumEndpoint")]
        fn remove_ethereum_endpoint(&self, String) -> Result<bool>;

        #[rpc(name="admin_ethereumEndpoints")]
        fn ethereum_endpoints(&self) -> Result<Vec<String>>;

        #[rpc(name="admin_ethereumEndpointCount")]
        fn ethereum_endpoint_count(&self) -> Result<usize>;
    }
}
